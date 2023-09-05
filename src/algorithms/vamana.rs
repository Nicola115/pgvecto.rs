use super::impls::vamana::VamanaImpl;
use super::Algo;
use crate::bgworker::index::IndexOptions;
use crate::bgworker::storage::Storage;
use crate::bgworker::storage::StoragePreallocator;
use crate::bgworker::vectors::Vectors;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum VamanaError {
    //
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VamanaOptions {
    #[serde(default = "VamanaOptions::default_memmap")]
    pub memmap: Memmap,
    /// out degree bound
    #[serde(default = "VamanaOptions::default_r")]
    pub r: usize,
    /// Distance threshold
    #[serde(default = "VamanaOptions::default_alpha")]
    pub alpha: f32,
    /// Search list size
    #[serde(default = "VamanaOptions::default_l")]
    pub l: usize,
}

impl VamanaOptions {
    fn default_memmap() -> Memmap {
        Memmap::Ram
    }
    fn default_r() -> usize {
        50
    }
    fn default_alpha() -> f32 {
        1.2
    }
    fn default_l() -> usize {
        70
    }
}

pub struct Vamana<D: DistanceFamily> {
    implementation: VamanaImpl<D>,
}

impl<D: DistanceFamily> Algo for Vamana<D> {
    type Error = VamanaError;

    type Save = ();

    fn prebuild(
        storage: &mut StoragePreallocator,
        options: IndexOptions,
    ) -> Result<(), Self::Error> {
        let vamana_options = options.algorithm.clone().unwrap_vamana();
        VamanaImpl::<D>::prebuild(
            storage,
            options.capacity,
            vamana_options.r,
            vamana_options.memmap,
        )?;
        Ok(())
    }

    fn build(
        storage: &mut Storage,
        options: IndexOptions,
        vectors: Arc<Vectors>,
        n: usize,
    ) -> Result<Self, VamanaError> {
        let vamana_options = options.algorithm.clone().unwrap_vamana();
        let implementation = VamanaImpl::new(
            storage,
            vectors,
            options.capacity,
            vamana_options.r,
            vamana_options.alpha,
            vamana_options.l,
            vamana_options.memmap,
        )?;
        Ok(Self { implementation })
    }

    fn load(
        storage: &mut Storage,
        options: IndexOptions,
        vectors: Arc<Vectors>,
        (): (),
    ) -> Result<Self, VamanaError> {
        let vamana_options = options.algorithm.unwrap_vamana();
        let implementation = VamanaImpl::load(
            storage,
            vectors,
            options.capacity,
            vamana_options.r,
            vamana_options.alpha,
            vamana_options.l,
            vamana_options.memmap,
        )?;
        Ok(Self { implementation })
    }
    fn insert(&self, insert: usize) -> Result<(), VamanaError> {
        // TODO: the insert API is a fake insert for user,
        // but can be used to implement concurrent index building
        Ok(())
    }
    fn search<F>(
        &self,
        target: Box<[Scalar]>,
        k: usize,
        filter: F,
    ) -> Result<Vec<(Scalar, u64)>, VamanaError>
    where
        F: FnMut(u64) -> bool,
    {
        self.implementation.search(target, k, filter)
    }
}
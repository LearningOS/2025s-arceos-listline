use hashbrown::HashMap as BaseHashMap;
use hashbrown::hash_map::Iter as BaseIter;
use core::hash::Hash;
use core::hash::BuildHasher;
use arceos_api::ax_random;

// 定义一个更高效的RandomState
pub struct RandomState {
    seed: u64,
}

impl RandomState {
    pub fn new() -> Self {
        // 使用arceos_api的ax_random函数生成随机种子
        let random_value = ax_random();
        // 使用一个简单的方式从u128中提取u64
        let seed = (random_value & 0xFFFFFFFFFFFFFFFF) as u64;
        Self { seed }
    }
}

impl Default for RandomState {
    fn default() -> Self {
        Self::new()
    }
}

// 使用FNV哈希算法实现，这是一种简单但高效的哈希算法
impl BuildHasher for RandomState {
    type Hasher = FnvHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FnvHasher::with_seed(self.seed)
    }
}

// FNV哈希算法实现
pub struct FnvHasher {
    hash: u64,
}

impl FnvHasher {
    // FNV-1a哈希算法的常量
    const FNV_PRIME: u64 = 1099511628211;
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;

    fn with_seed(seed: u64) -> Self {
        Self { hash: Self::FNV_OFFSET_BASIS ^ seed }
    }
}

impl core::hash::Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.hash ^= b as u64;
            self.hash = self.hash.wrapping_mul(Self::FNV_PRIME);
        }
    }
}

// 自定义HashMap迭代器
pub struct Iter<'a, K, V> {
    base: BaseIter<'a, K, V>,
}

// 为Iter实现Iterator trait
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
}

pub struct HashMap<K, V, S = RandomState> {
    base: BaseHashMap<K, V, S>,
}

impl<K, V> HashMap<K, V, RandomState> {
    #[inline]
    pub fn new() -> HashMap<K, V, RandomState> {
        HashMap {
            base: BaseHashMap::with_hasher(RandomState::new())
        }
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter { base: self.base.iter() }
    }
    
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.base.insert(k, v)
    }
}
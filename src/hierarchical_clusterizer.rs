use std::collections::{BTreeMap, BinaryHeap};
use std::cmp::{Reverse, Ordering};


pub trait ClusterTrait {
    fn merge(cluster1: &Self, cluster2: &Self) -> Self;
    fn distance(cluster1: &Self, cluster2: &Self) -> Option<f64>;
}


#[derive(Debug, PartialEq, PartialOrd)]
struct DistanceHeapNode {
    distance: f64,
    cluster_ids: (usize, usize),
}


impl Ord for DistanceHeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.distance > other.distance {
            Ordering::Greater
        } else if self.distance > other.distance {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}


impl Eq for DistanceHeapNode {}


pub struct HierarchicalClusterizer<T: ClusterTrait> {
    opt_1d: bool,
    cluster_map: BTreeMap<usize, T>,
    cluster_id_next: usize,
    distance_heap: BinaryHeap<Reverse<DistanceHeapNode>>,
    order_map: BTreeMap<usize, (Option<usize>, Option<usize>)>,
}


impl<T: ClusterTrait + Clone> HierarchicalClusterizer<T> {
    pub fn new() -> Self {
        Self {
            opt_1d: false,
            cluster_map: BTreeMap::new(),
            cluster_id_next: 0,
            distance_heap: BinaryHeap::new(),
            order_map: BTreeMap::new(),
        }
    }

    pub fn set_1d_opt(&mut self, opt_1d: bool) {
        self.opt_1d = opt_1d;
    }

    pub fn initialize(&mut self, clusters: &[T]) {
        // Build cluster_map
        self.cluster_map.clear();
        self.cluster_map.extend(
            clusters.iter().cloned().enumerate()
        );

        // Set next cluster_id
        self.cluster_id_next = self.cluster_map.len();

        // Build distance_heap
        if self.opt_1d {
            self._build_order_map();
            self._build_distance_heap_1d();
        } else {
            self._build_distance_heap();
        }
    }

    fn _build_distance_heap(&mut self) {
        let size = self.cluster_map.len();

        self.distance_heap.clear();

        for cluster_id1 in 0..size {
            for cluster_id2 in (cluster_id1 + 1)..size {
                if let Some(node) = self._create_distance_heap_node(
                            cluster_id1, cluster_id2
                        ) {
                    self.distance_heap.push(node);
                }
            }
        }
    }

    fn _build_distance_heap_1d(&mut self) {
        let size = self.cluster_map.len();

        self.distance_heap.clear();

        for cluster_id1 in 0..(size - 1) {
            let cluster_id2 = cluster_id1 + 1;

            if let Some(node) = self._create_distance_heap_node(
                        cluster_id1, cluster_id2
                    ) {
                self.distance_heap.push(node);
            }
        }
    }

    fn _build_order_map(&mut self) {
        let size = self.cluster_map.len();

        self.order_map.clear();

        for cluster_id in 0..size {
            self.order_map.insert(
                cluster_id,
                (
                    if cluster_id > 0 { Some(cluster_id - 1) } 
                                 else { None },
                    if cluster_id < size - 1 { Some(cluster_id + 1) } 
                                        else { None },
                )
            );
        }
    }

    pub fn cluster_count(&self) -> usize {
        self.cluster_map.len()
    }

    pub fn get_clusters(&self) -> Vec<T> {
        self.cluster_map.values().cloned().collect()
    }

    fn _distance_heap_pop(&mut self) -> Option<(usize, usize)> {
        loop {
            let node = self.distance_heap.pop();

            if let Some(Reverse(DistanceHeapNode { cluster_ids, .. })) = node {
                if self.cluster_map.contains_key(&cluster_ids.0) && 
                   self.cluster_map.contains_key(&cluster_ids.1) {
                    return Some(cluster_ids);
                }
            } else {
                return None;
            }
        }
    }

    fn _create_distance_heap_node(&self, cluster_id1: usize, 
                                  cluster_id2: usize) -> 
                Option<Reverse<DistanceHeapNode>> {
        if let Some(distance) = T::distance(
                    &self.cluster_map[&cluster_id1], 
                    &self.cluster_map[&cluster_id2]
                ) {
            Some(Reverse(DistanceHeapNode {
                distance,
                cluster_ids: (cluster_id1, cluster_id2),
            }))
        } else {
            None
        }
    }

    fn _distance_heap_push_related(&mut self, cluster_new_id: usize) {
        for &cluster_id in self.cluster_map.keys() {
            if cluster_id != cluster_new_id {
                if let Some(node) = self._create_distance_heap_node(
                            cluster_id, cluster_new_id
                        ) {
                    self.distance_heap.push(node);
                }
            }
        }
    }

    fn _distance_heap_push_related_1d(&mut self, cluster_new_id: usize) {
        // Left
        if let Some(cluster_id) = self.order_map[&cluster_new_id].0 {
            if let Some(node) = self._create_distance_heap_node(
                        cluster_id, cluster_new_id
                    ) {
                self.distance_heap.push(node);
            }
        }

        // Right
        if let Some(cluster_id) = self.order_map[&cluster_new_id].1 {
            if let Some(node) = self._create_distance_heap_node(
                        cluster_new_id, cluster_id
                    ) {
                self.distance_heap.push(node);
            }
        }
    }

    fn _order_map_insert_new(&mut self, cluster_new_id: usize, 
                             cluster_id1: usize, cluster_id2: usize) {
        // cluster_id1 goes before cluster_id2 in the linked list
        // Insert new node
        self.order_map.insert(
            cluster_new_id, 
            (
                self.order_map[&cluster_id1].0, 
                self.order_map[&cluster_id2].1,
            )
        );

        // Rebind left node
        if let Some(cluster_id) = self.order_map[&cluster_new_id].0 {
            self.order_map.get_mut(&cluster_id).unwrap().1 = 
                Some(cluster_new_id);
        } 

        // Rebind right node
        if let Some(cluster_id) = self.order_map[&cluster_new_id].1 {
            self.order_map.get_mut(&cluster_id).unwrap().0 = 
                Some(cluster_new_id);
        } 

        // Remove old nodes
        self.order_map.remove(&cluster_id1);
        self.order_map.remove(&cluster_id2);
    }

    fn _merge_clusters(&mut self, cluster_id1: usize, 
                       cluster_id2: usize) -> usize {
        let cluster_new = T::merge(&self.cluster_map[&cluster_id1], 
                                   &self.cluster_map[&cluster_id2]);

        let cluster_new_id = self.cluster_id_next;
        self.cluster_map.insert(cluster_new_id, cluster_new);
        self.cluster_id_next += 1;

        cluster_new_id
    }

    pub fn step(&mut self) -> bool {
        // 1. Get closest clusters
        let cluster_id_pair = self._distance_heap_pop();
        
        if let Some((cluster_id1, cluster_id2)) = cluster_id_pair {
            // 2. Merge clusters
            let cluster_new_id = self._merge_clusters(cluster_id1, cluster_id2);

            // 3. Remove old clusters
            self.cluster_map.remove(&cluster_id1);
            self.cluster_map.remove(&cluster_id2);

            // 4. Calculate distances to related clusters
            if self.opt_1d {
                self._order_map_insert_new(
                    cluster_new_id, cluster_id1, cluster_id2
                );
                self._distance_heap_push_related_1d(cluster_new_id);
            } else {
                self._distance_heap_push_related(cluster_new_id);
            }

            // Return
            true
        } else {
            false
        }
    }

    pub fn clusterize(&mut self, final_cluster_count: usize) -> usize {
        let mut cluster_count = self.cluster_count();
        
        while final_cluster_count < cluster_count {
            let success = self.step();
            if success {
                cluster_count -= 1;
            } else {
                break;
            }
        }

        cluster_count
    }
}


#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use test::Bencher;
    use rand::Rng;

    #[derive(Debug, Clone, PartialEq)]
    struct Cluster {
        x: f64,
        y: f64,
        w: f64,
    }

    impl ClusterTrait for Cluster {
        fn merge(cluster1: &Self, cluster2: &Self) -> Self {
            let w = cluster1.w + cluster2.w;
            let x = (cluster1.x * cluster1.w + cluster2.x * cluster2.w) / w;
            let y = (cluster1.y * cluster1.w + cluster2.y * cluster1.w) / w;
            Self { x, y, w }
        }

        fn distance(cluster1: &Self, cluster2: &Self) -> Option<f64> {
            Some(
                (cluster1.x - cluster2.x) * (cluster1.x - cluster2.x) +
                (cluster1.y - cluster2.y) * (cluster1.y - cluster2.y)
            )
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Cluster1d {
        x: f64,
        w: f64,
    }

    impl ClusterTrait for Cluster1d {
        fn merge(cluster1: &Self, cluster2: &Self) -> Self {
            let w = cluster1.w + cluster2.w;
            let x = (cluster1.x * cluster1.w + cluster2.x * cluster2.w) / w;
            Self { x, w }
        }

        fn distance(cluster1: &Self, cluster2: &Self) -> Option<f64> {
            let z = (cluster1.x - cluster2.x) * (cluster1.x - cluster2.x);
            if z < 0.002 { Some(z) } else { None }
        }
    }

    #[test]
    fn test_2d() {
        let clusters = vec![
            Cluster { x: 0.0, y: 0.0, w: 1.0 },
            Cluster { x: 10.0, y: 10.0, w: 1.0 },
            Cluster { x: 11.0, y: 11.0, w: 1.0 },
            Cluster { x: 4.0, y: 4.0, w: 1.0 },
            Cluster { x: 1.0, y: 1.0, w: 1.0 },
        ];

        let mut clusterizer = HierarchicalClusterizer::<Cluster>::new();
        clusterizer.initialize(&clusters);

        assert_eq!(clusterizer.cluster_count(), 5);

        clusterizer.step();
        clusterizer.step();

        assert_eq!(clusterizer.cluster_count(), 3);

        clusterizer.step();
        clusterizer.step();

        assert_eq!(clusterizer.cluster_count(), 1);

        assert_eq!(clusterizer.get_clusters()[0], 
                   Cluster { x: 5.2, y: 4.8, w: 5.0 });
    }

    #[test]
    fn test_1d() {
        let clusters = vec![
            Cluster1d { x: 0.0, w: 1.0 },
            Cluster1d { x: 0.01, w: 1.0 },
            Cluster1d { x: 0.04, w: 1.0 },
            Cluster1d { x: 0.1,  w: 1.0 },
            Cluster1d { x: 0.11,  w: 1.0 },
        ];

        let mut clusterizer = HierarchicalClusterizer::<Cluster1d>::new();
        clusterizer.set_1d_opt(true);
        clusterizer.initialize(&clusters);

        assert_eq!(clusterizer.clusterize(1), 2);
    }

    #[bench]
    fn bench_2d(bencher: &mut Bencher) {
        let size = 100;
        let mut rng = rand::thread_rng();
        let clusters: Vec<Cluster> = (0..size)
            .map(|_| Cluster { x: rng.gen(), y: rng.gen(), w: 1.0 })
            .collect();
        bencher.iter(|| {
            let mut clusterizer = HierarchicalClusterizer::<Cluster>::new();
            clusterizer.initialize(&clusters);
            clusterizer.clusterize(1);
        });
    }

    #[bench]
    fn bench_1d(bencher: &mut Bencher) {
        let size = 100;
        let mut rng = rand::thread_rng();
        let mut clusters: Vec<Cluster1d> = (0..size)
            .map(|_| Cluster1d { x: rng.gen(), w: 1.0 })
            .collect();
        clusters.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        bencher.iter(|| {
            let mut clusterizer = HierarchicalClusterizer::<Cluster1d>::new();
            clusterizer.set_1d_opt(true);
            clusterizer.initialize(&clusters);
            clusterizer.clusterize(1);
        });
    }
}

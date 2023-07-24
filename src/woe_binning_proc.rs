use std::collections::HashMap;

use crate::hierarchical_clusterizer::{ClusterTrait, HierarchicalClusterizer};


pub fn calc_woe(p0: f64, p1: f64) -> f64 {
    (p0 / p1).ln()
}


pub fn calc_iv(p0: f64, p1: f64) -> f64 {
    calc_woe(p0, p1) * (p0 - p1)
}


#[derive(Debug, Clone, PartialEq)]
struct WoeBinningCluster {
    n: usize,
    n1: usize,
    n0: usize,
    p1: f64,
    p0: f64,
    values: Vec<usize>,
    smooth: f64,
}


impl ClusterTrait for WoeBinningCluster {
    fn merge(cluster1: &Self, cluster2: &Self) -> Self {
        let smooth = (cluster1.n as f64 * cluster1.smooth + 
                      cluster2.n as f64 * cluster2.smooth) / 
                     (cluster1.n + cluster2.n) as f64;
        Self {
            n: cluster1.n + cluster2.n,
            n1: cluster1.n1 + cluster2.n1,
            n0: cluster1.n0 + cluster2.n0,
            p1: cluster1.p1 + cluster2.p1,
            p0: cluster1.p0 + cluster2.p0,
            values: [cluster1.values.clone(), cluster2.values.clone()].concat(),
            smooth,
        }
    }

    fn distance(cluster1: &Self, cluster2: &Self) -> Option<f64> {
        let smooth = (cluster1.n as f64 * cluster1.smooth + 
                      cluster2.n as f64 * cluster2.smooth) / 
                     (cluster1.n + cluster2.n) as f64;

        let d_iv = calc_iv(cluster1.p0, cluster1.p1) + 
                   calc_iv(cluster2.p0, cluster2.p1) - 
                   calc_iv(cluster1.p0 + cluster2.p0, 
                           cluster1.p1 + cluster2.p1);

        let factor = (cluster1.n * cluster2.n) as f64;

        Some(d_iv * factor.powf(smooth))
    }
}


pub struct WoeBinningProc {
    clusterizer: HierarchicalClusterizer<WoeBinningCluster>,
    desirable_bins_num: usize,
    clusters: Option<Vec<WoeBinningCluster>>,
    smooth: f64,
}


impl WoeBinningProc {
    pub fn new(desirable_bins_num: usize, smooth: f64) -> Self {
        Self {
            clusterizer: HierarchicalClusterizer::<WoeBinningCluster>::new(),
            desirable_bins_num,
            clusters: None,
            smooth,
        }
    }

    pub fn process_categorial(&mut self, series: &[usize], target: &[bool]) {
        assert_eq!(self.clusters, None);
        assert_eq!(series.len(), target.len());

        let initial_clusters = self._collect_clusters(series, target);

        self.clusterizer.initialize(&initial_clusters);
        self.clusterizer.clusterize(self.desirable_bins_num);
        self.clusters = Some(self.clusterizer.get_clusters());
    }

    pub fn process_numeric(&mut self, series: &[usize], target: &[bool]) {
        assert_eq!(self.clusters, None);
        assert_eq!(series.len(), target.len());

        let mut initial_clusters = self._collect_clusters(series, target);
        initial_clusters.sort_by(
            |a, b| a.values[0].partial_cmp(&b.values[0]).unwrap()
        );
        
        self.clusterizer.set_1d_opt(true);
        self.clusterizer.initialize(&initial_clusters);
        self.clusterizer.clusterize(self.desirable_bins_num);
        self.clusters = Some(self.clusterizer.get_clusters());
    }

    pub fn is_done(&self) -> bool {
        self.clusters.is_some()
    }

    pub fn get_bins_num(&self) -> Option<usize> {
        self.clusters.as_ref().map(|c| c.len())
    }

    pub fn get_bins_array(&self) -> Option<Vec<Vec<usize>>> {
        self.clusters.as_ref().map(|v| 
            v.iter().map(|c| c.values.clone()).collect()
        )
    }

    pub fn get_size_array(&self) -> Option<Vec<usize>> {
        self.clusters.as_ref().map(|v| 
            v.iter().map(|c| c.values.len()).collect()
        )
    }

    pub fn get_bin_values(&self, idx: usize) -> Option<Vec<usize>> {
        self.clusters.as_ref().map(|v| 
            v[idx].values.clone()
        )
    }

    pub fn get_woe_array(&self) -> Option<Vec<f64>> {
        self.clusters.as_ref().map(|v| 
            v.iter().map(|c| calc_woe(c.p0, c.p1)).collect()
        )
    }

    pub fn get_iv_array(&self) -> Option<Vec<f64>> {
        self.clusters.as_ref().map(|v| 
            v.iter().map(|c| calc_iv(c.p0, c.p1)).collect()
        )
    }

    pub fn get_iv_total(&self) -> Option<f64> {
        self.get_iv_array().map(|v| v.iter().sum())
    }

    fn _collect_clusters(&self, series: &[usize], target: &[bool]) -> 
                Vec<WoeBinningCluster> {
        // Statistics for values in series
        let mut stat = HashMap::new();

        // Counts of 1 and 0 targets
        let mut c1 = 0;
        let mut c0 = 0;

        // Collect statistics
        for idx in 0..series.len() {
            let value = series[idx];

            if !stat.contains_key(&value) {
                stat.insert(
                    value, 
                    WoeBinningCluster {
                        n: 0,
                        n1: 0,
                        n0: 0,
                        p1: 0.0,
                        p0: 0.0,
                        values: vec![value],
                        smooth: self.smooth,
                    }
                );
            }

            let cluster = stat.get_mut(&value).unwrap();
            cluster.n += 1;
            if target[idx] {
                cluster.n1 += 1;
                c1 += 1;
            } else {
                cluster.n0 += 1;
                c0 += 1;
            }
        }

        // Calculate parts goods and bads for each value (using Laplas formula)
        for (_, cluster) in stat.iter_mut() {
            cluster.p1 = (cluster.n1 + 1) as f64 / (c1 + 2) as f64;
            cluster.p0 = (cluster.n0 + 1) as f64 / (c0 + 2) as f64;
        }

        // Return clusters as a vector
        stat.into_values().collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    use crate::binary_search;

    const SEED: u64 = 0;

    fn create_rng() -> impl Rng {
        StdRng::seed_from_u64(SEED)
    }

    fn build_cat_map<R: Rng>(rng: &mut R, cat_num: usize) -> Vec<f64> {
        let mut cat_map = Vec::new();
        let mut agg_sum = 0.0;
        for _ in 0..cat_num {
            let prob = rng.gen::<f64>();
            agg_sum += prob;
            cat_map.push(agg_sum);
        }
        for idx in 0..cat_num {
            cat_map[idx] /= agg_sum;
        }
        cat_map
    }


    fn build_prob_map<R: Rng>(rng: &mut R, cat_num: usize, 
                              sorted: bool) -> Vec<f64> {
        let mut prob_map = Vec::new();
        for _ in 0..cat_num {
            let prob = rng.gen::<f64>();
            prob_map.push(prob);
        }
        if sorted {
            prob_map.sort_by(|a, b| a.partial_cmp(&b).unwrap())
        }
        prob_map
    }

    fn create_dataset(size: usize, cat_num: usize, sorted: bool) -> 
                      (Vec<usize>, Vec<bool>) {
        let mut rng = create_rng();

        let prob_map = build_prob_map(&mut rng, cat_num, sorted);
        let cat_map = build_cat_map(&mut rng, cat_num);

        let mut series = Vec::new();
        let mut target = Vec::new();
        for _ in 0..size {
            let cat = binary_search(cat_num, &cat_map, rng.gen::<f64>());
            let trg = rng.gen::<f64>() < prob_map[cat];
            series.push(cat);
            target.push(trg);
        }
        (series, target)
    }

    #[test]
    fn test_categorial() {
        let (series, target) = create_dataset(1_000, 10, false);

        let mut wbp = WoeBinningProc::new(4);
        wbp.process_categorial(&series, &target);

        assert_eq!(wbp.is_done(), true);
        assert_eq!(wbp.get_bins_num(), Some(4));
        assert_eq!(wbp.get_iv_total(), Some(1.1660067107931673));
    }

    #[test]
    fn test_numeric() {
        let (series, target) = create_dataset(1_000, 10, true);

        let mut wbp = WoeBinningProc::new(4);
        wbp.process_numeric(&series, &target);

        assert_eq!(wbp.is_done(), true);
        assert_eq!(wbp.get_bins_num(), Some(4));
        assert_eq!(wbp.get_iv_total(), Some(2.3937109658539946));
    }
}

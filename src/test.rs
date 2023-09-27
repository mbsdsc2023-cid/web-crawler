#[test]
fn bfs() {
    use crate::crawler::{AdjacentNodes, Crawler};

    struct AdjVec(Vec<Vec<usize>>);

    impl AdjacentNodes for AdjVec {
        type Node = usize;

        fn adjacent_nodes(&self, v: &Self::Node) -> Vec<Self::Node> {
            self.0.get(*v).cloned().unwrap_or(Vec::new())
        }
    }

    let graph = AdjVec(vec![vec![1, 2], vec![0, 3], vec![3], vec![2, 0]]);
    let crawler = Crawler::new(&graph, 0);
    let nodes: Vec<usize> = crawler.collect();

    assert_eq!(nodes, vec![0, 1, 2, 3]);
}

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::time::SystemTime;
use eyre::{anyhow, eyre};
use itertools::Itertools;
use tracing::{debug, info, Instrument, Level, span, trace};
use crate::days::Day;

const DAY: Day = Day(5);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Graph {
    sorted_nodes: Vec<u8>,
    node_mapping: Vec<u8>,
    adjacency_matrix: Vec<bool>,
}

impl Display for Graph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(&self.sorted_nodes)
            .finish()?;
        writeln!(f)?;
        let matrix = self.adjacency_matrix.iter()
            .chunks(self.sorted_nodes.len())
            .into_iter()
            .map(|row|
                row.map(|&adjacent| if adjacent { '1' } else { '0' }).join(" ")
            )
            .join("\n");
        write!(f, "{matrix}")
    }
}

impl Graph {
    pub fn new(edges: &[(u8, u8)]) -> Self {
        let sorted_nodes: Vec<u8> = edges.iter()
            .map(|(from, to)| [from, to])
            .flatten()
            .cloned()
            .unique()
            .sorted()
            .collect();
        let node_mapping = (0..*sorted_nodes.last().unwrap())
            .map(|node| sorted_nodes.binary_search(&node).unwrap_or(0) as u8)
            .collect::<Vec<_>>();
        let node_count = sorted_nodes.len();
        let edges = edges.iter()
            .map(|(from, to)| (sorted_nodes.binary_search(from).unwrap(), sorted_nodes.binary_search(to).unwrap()))
            .collect::<Vec<_>>();

        let mut adjacency_matrix = vec![false; node_count * node_count];
        for &(from, to) in &edges {
            adjacency_matrix[from * node_count + to] = true;
        }

        let edge_count = adjacency_matrix.iter().filter(|&&adjacent| adjacent).count();
        assert_eq!(edge_count, edges.len());

        Self { sorted_nodes, node_mapping, adjacency_matrix }
    }

    pub fn add_transitive(&mut self) {
        let node_count = self.sorted_nodes.len();
        let without_predecessor = (0..node_count)
            .map(|node| {
                (node, self.predecessors(node).len())
            })
            .filter(|(_, count)| *count == 0)
            .map(|(node, _)| node)
            .collect::<Vec<_>>();

        for node in without_predecessor {
            let successors = self.successors(node);
            for successor in successors {
                self.add_transitive_for_node(successor);
            }
        }
    }

    fn successors(&self, node: usize) -> Vec<usize> {
        let node_count = self.sorted_nodes.len();
        (&self.adjacency_matrix[node * node_count..node * (node_count + 1)]).iter()
            .enumerate()
            .filter(|&(_, &adjacent)| adjacent)
            .map(|(node, _)| node)
            .collect()
    }

    fn predecessors(&self, node: usize) -> Vec<usize> {
        let node_count = self.sorted_nodes.len();
        self.adjacency_matrix.iter()
            .skip(node)
            .step_by(node_count)
            .enumerate()
            .filter(|&(_, &adjacent)| adjacent)
            .map(|(node, _)| node)
            .collect()
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.adjacency_matrix[from * self.sorted_nodes.len() + to] = true;
    }

    fn has_edge(&self, from: usize, to: usize) -> bool {
        self.adjacency_matrix[from * self.sorted_nodes.len() + to]
    }

    fn add_transitive_for_node(&mut self, node: usize) {
        let predecessors = self.predecessors(node);
        for successor in self.successors(node) {
            for &predecessor in &predecessors {
                self.add_edge(predecessor, successor);
            }
            self.add_transitive_for_node(successor);
        }
    }
}

pub fn parse(input: &str) -> eyre::Result<Input> {
    let (raw_rules, raw_manual) = input.split_once("\n\n")
        .ok_or(eyre!("Failed to split rules and manuals"))?;

    let rules = raw_rules.lines()
        .map(|line| {
            let (prior, posterior) = line.split_once('|')
                .ok_or(anyhow!("Failed to split rule at `|`"))?;
            let prior: u8 = prior.parse()?;
            let posterior: u8 = posterior.parse()?;
            Ok((prior, posterior))
        })
        .collect::<eyre::Result<Vec<_>>>()?;

    let manuals: Vec<Vec<u8>> = raw_manual.lines()
        .map(|line| line
            .split(',')
            .map(|page| page.parse()
                .map_err(|err: ParseIntError| err.into())
            )
            .collect::<eyre::Result<_>>()
        )
        .collect::<eyre::Result<_>>()?;

    let result = manuals.into_iter()
        .map(|manual| {
            let lookup: HashSet<u8> = HashSet::from_iter(manual.iter().cloned());
            let rules = rules.iter()
                .filter(|(from, to)| lookup.contains(from) && lookup.contains(to))
                .cloned()
                .collect::<Vec<_>>();
            let graph = Graph::new(&rules);
            (graph, manual)
        })
        .collect();

    Ok(result)
}

type Input = Vec<(Graph, Vec<u8>)>;

pub fn process_part1(input: &Input) -> eyre::Result<String> {
    let result: usize = input.iter()
        .map(|(graph, manuals)| {
            let mut graph = graph.clone();
            graph.add_transitive();
            let mut pages = (0..graph.sorted_nodes.len()).collect::<Vec<_>>();
            pages.sort_by(|&a, &b|
                if graph.has_edge(a, b) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                });
            let pages = pages.iter()
                .map(|&node| graph.sorted_nodes[node])
                .collect::<Vec<_>>();
            trace!("{pages:?}");
            (pages, manuals)
        })
        .filter(|(pages, manuals)| pages.eq(*manuals))
        .map(|(pages, _)| pages[pages.len()/2] as usize)
        .sum();

    Ok(result.to_string())
}

pub fn process_part2(input: &Input) -> eyre::Result<String> {
    let result: usize = input.iter()
        .map(|(graph, manuals)| {
            let mut graph = graph.clone();
            graph.add_transitive();
            let mut pages = (0..graph.sorted_nodes.len()).collect::<Vec<_>>();
            pages.sort_by(|&a, &b|
                if graph.has_edge(a, b) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                });
            let pages = pages.iter()
                .map(|&node| graph.sorted_nodes[node])
                .collect::<Vec<_>>();
            trace!("{pages:?}");
            (pages, manuals)
        })
        .filter(|(pages, manuals)| !pages.eq(*manuals))
        .map(|(pages, _)| pages[pages.len()/2] as usize)
        .sum();

    Ok(result.to_string())
}

pub async fn run() -> eyre::Result<()> {
    let day_span = span!(Level::ERROR, "", "{}", DAY);
    async {
        info!("Running {DAY}");

        let raw_input = super::get_input(DAY).await?;
        trace!(raw_input);

        let input = parse(&raw_input)?;
        debug!(?input);

        let start1 = SystemTime::now();
        let result1 = process_part1(&input)?;
        let end1 = SystemTime::now();
        let start2 = SystemTime::now();
        let result2 = process_part2(&input)?;
        let end2 = SystemTime::now();
        println!("{DAY} result:");
        println!("  part 1: {result1} in {:?}", end1.duration_since(start1).unwrap());
        println!("  part 2: {result2} in {:?}", end2.duration_since(start2).unwrap());
        Ok(())
    }
        .instrument(day_span.or_current())
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_example() {
        let raw_input = r#"47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47
"#;
        let input = parse(&raw_input).unwrap();

        let result1 = process_part1(&input).unwrap();
        assert_eq!("143", result1);

        let result2 = process_part2(&input).unwrap();
        assert_eq!("123", result2);
    }
}

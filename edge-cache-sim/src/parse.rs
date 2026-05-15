//! Comma-separated list parsing for CLI sweeps.

pub fn comma_separated_usize(s: &str) -> anyhow::Result<Vec<usize>> {
    let s = s.trim();
    if s.is_empty() {
        anyhow::bail!("empty list");
    }
    s.split(',')
        .map(|p| p.trim().parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("invalid usize list: {e}"))
}

pub fn comma_separated_f64(s: &str) -> anyhow::Result<Vec<f64>> {
    let s = s.trim();
    if s.is_empty() {
        anyhow::bail!("empty list");
    }
    s.split(',')
        .map(|p| p.trim().parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("invalid f64 list: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spaces() {
        assert_eq!(comma_separated_usize("10, 20 ,30").unwrap(), vec![10, 20, 30]);
    }

    #[test]
    fn parses_f64() {
        assert_eq!(comma_separated_f64("0, 0.05 ,1").unwrap(), vec![0.0, 0.05, 1.0]);
    }
}

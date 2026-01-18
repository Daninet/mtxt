use crate::{BeatTime, types::beat_fraction::BeatFraction};
use anyhow::{Result, bail};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum BeatValue {
    Time(BeatTime),
    Fraction(BeatFraction),
}

impl BeatValue {
    pub fn as_beat_time(&self) -> BeatTime {
        match self {
            BeatValue::Time(t) => *t,
            BeatValue::Fraction(f) => f.as_beat_time(),
        }
    }
}

impl fmt::Display for BeatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BeatValue::Time(t) => write!(f, "{}", t),
            BeatValue::Fraction(fr) => write!(f, "{}", fr),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatOperator {
    Plus,
    Minus,
    Multiply,
}

impl fmt::Display for BeatOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BeatOperator::Plus => write!(f, "+"),
            BeatOperator::Minus => write!(f, "-"),
            BeatOperator::Multiply => write!(f, "*"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BeatExpressionItem {
    Value(BeatValue),
    Operator(BeatOperator),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeatExpression {
    items: Vec<BeatExpressionItem>,
}

impl BeatExpression {
    fn evaluate_sums(&self) -> (BeatTime, BeatTime) {
        if self.items.is_empty() {
            return (BeatTime::zero(), BeatTime::zero());
        }

        let mut pos_sum = BeatTime::zero();
        let mut neg_sum = BeatTime::zero();

        let mut current_term: Option<BeatTime> = None;
        let mut current_op = BeatOperator::Plus;

        for item in &self.items {
            match item {
                BeatExpressionItem::Value(v) => {
                    let vt = v.as_beat_time();
                    if let Some(ct) = current_term {
                        current_term = Some(ct * vt);
                    } else {
                        current_term = Some(vt);
                    }
                }
                BeatExpressionItem::Operator(op) => match op {
                    BeatOperator::Plus | BeatOperator::Minus => {
                        if let Some(ct) = current_term {
                            match current_op {
                                BeatOperator::Plus => pos_sum = pos_sum + ct,
                                BeatOperator::Minus => neg_sum = neg_sum + ct,
                                _ => unreachable!(),
                            }
                        }
                        current_term = None;
                        current_op = *op;
                    }
                    BeatOperator::Multiply => {}
                },
            }
        }

        if let Some(ct) = current_term {
            match current_op {
                BeatOperator::Plus => pos_sum = pos_sum + ct,
                BeatOperator::Minus => neg_sum = neg_sum + ct,
                _ => unreachable!(),
            }
        }

        (pos_sum, neg_sum)
    }

    pub fn as_beat_time(&self) -> BeatTime {
        let (pos, neg) = self.evaluate_sums();
        pos - neg
    }

    pub fn value(&self) -> f64 {
        self.as_beat_time().as_f64()
    }
}

impl FromStr for BeatExpression {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            bail!("Empty expression");
        }

        if s.contains(' ') {
            bail!("Spaces are not allowed in beat expressions");
        }

        let mut items = Vec::new();
        let mut current = String::new();

        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '+' || c == '-' || c == '*' {
                if !current.is_empty() {
                    items.push(BeatExpressionItem::Value(parse_beat_value(&current)?));
                    current.clear();
                }
                let op = match c {
                    '+' => BeatOperator::Plus,
                    '-' => BeatOperator::Minus,
                    '*' => BeatOperator::Multiply,
                    _ => unreachable!(),
                };
                items.push(BeatExpressionItem::Operator(op));
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() {
            items.push(BeatExpressionItem::Value(parse_beat_value(&current)?));
        }

        // Validate rules
        // 1. Multiplication operands must be explicit fractions
        for i in 0..items.len() {
            if let BeatExpressionItem::Operator(BeatOperator::Multiply) = items[i] {
                // Check previous
                if i == 0 || i == items.len() - 1 {
                    bail!("Multiply operator at the start or end of expression");
                }
                if let BeatExpressionItem::Value(BeatValue::Time(t)) = &items[i - 1] {
                    bail!("Multiplication operands must be explicit fractions: {}", t);
                }
                if let BeatExpressionItem::Value(BeatValue::Time(t)) = &items[i + 1] {
                    bail!("Multiplication operands must be explicit fractions: {}", t);
                }
            }
        }

        let expr = Self { items };
        let (pos, neg) = expr.evaluate_sums();
        if pos < neg {
            bail!("Negative expression result: {}", expr.to_string());
        }

        Ok(expr)
    }
}

fn parse_beat_value(s: &str) -> Result<BeatValue> {
    if s.contains('/') {
        let frac: BeatFraction = s.parse()?;
        Ok(BeatValue::Fraction(frac))
    } else {
        let time: BeatTime = s.parse()?;
        Ok(BeatValue::Time(time))
    }
}

impl fmt::Display for BeatExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            match item {
                BeatExpressionItem::Value(v) => write!(f, "{}", v)?,
                BeatExpressionItem::Operator(op) => write!(f, "{}", op)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_expressions() {
        let cases = vec![
            ("1.25", 1.25),
            ("1.0+1/4", 1.25),
            ("1/4+1.0", 1.25),
            ("1/2*2/3", 1.0 / 3.0),
            ("9/5*5/7*9/11*7/13+2.3", 0.5664336 + 2.3),
            ("1.33+4/5*6/5+1.0", 1.33 + 0.96 + 1.0),
            ("2.0-1/4", 1.75),
            ("4/1*5/6", 20.0 / 6.0),
            ("1/3*2/5+5/7*7/11+11/13*13/17", 1.234937),
        ];

        for (input, expected) in cases {
            let expr: BeatExpression = input.parse().unwrap();
            assert!(
                (expr.value() - expected).abs() < 1e-6,
                "Failed for {}: got {}, expected {}",
                input,
                expr.value(),
                expected
            );
            let reconstructed = expr.to_string();
            assert_eq!(reconstructed, input);
        }
    }

    #[test]
    fn test_invalid_expressions() {
        let cases = vec!["2-4*5/6", "1.33+4.2*6/5", "1/2/3", "1 + 2", "1.5/2", "1-2"];

        for input in cases {
            assert!(
                input.parse::<BeatExpression>().is_err(),
                "Should have failed: {}",
                input
            );
        }
    }
}

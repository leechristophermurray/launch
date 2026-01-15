use crate::interface::ports::ICalculator;
use meval;

pub struct MevalCalculatorAdapter;

impl MevalCalculatorAdapter {
    pub fn new() -> Self {
        Self
    }
    
    fn preprocess_latex(&self, expression: &str) -> String {
        let mut expr = expression.to_string();
        
        // Basic Replacements
        expr = expr.replace("\\cdot", "*")
            .replace("\\times", "*")
            .replace("\\left(", "(")
            .replace("\\right)", ")");

        // Regex Replacements
        // \sqrt{...} -> sqrt(...)
        let sqrt_re = regex::Regex::new(r"\\sqrt\{([^}]+)\}").unwrap();
        expr = sqrt_re.replace_all(&expr, "sqrt($1)").to_string();
        
        // \frac{a}{b} -> (a)/(b)
        let frac_re = regex::Regex::new(r"\\frac\{([^}]+)\}\{([^}]+)\}").unwrap();
        expr = frac_re.replace_all(&expr, "($1)/($2)").to_string();
        
        // x^{y} -> x^(y) 
        // Note: meval often supports ^, but just in case
        let pow_re = regex::Regex::new(r"\^\{([^}]+)\}").unwrap();
        expr = pow_re.replace_all(&expr, "^($1)").to_string();

        expr
    }
}

impl ICalculator for MevalCalculatorAdapter {
    fn calculate(&self, expression: &str) -> Option<String> {
        let cleaned = self.preprocess_latex(expression);
        match meval::eval_str(&cleaned) {
            Ok(val) => Some(val.to_string()),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latex_preprocessing() {
        let adapter = MevalCalculatorAdapter::new();
        
        assert_eq!(adapter.preprocess_latex(r"\sqrt{16}"), "sqrt(16)");
        assert_eq!(adapter.preprocess_latex(r"\frac{1}{2}"), "(1)/(2)");
        assert_eq!(adapter.preprocess_latex(r"2 \times 3"), "2 * 3");
        assert_eq!(adapter.preprocess_latex(r"2^{3}"), "2^(3)");
        assert_eq!(adapter.preprocess_latex(r"\sqrt{9} + \frac{4}{2}"), "sqrt(9) + (4)/(2)");
    }
}

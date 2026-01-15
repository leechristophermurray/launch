// Calculator adapter with extensive LaTeX preprocessing
use crate::interface::ports::ICalculator;
use meval;
use regex::Regex;

pub struct MevalCalculatorAdapter;

impl MevalCalculatorAdapter {
    pub fn new() -> Self {
        Self
    }

    fn preprocess_latex(&self, expression: &str) -> String {
        let mut expr = expression.to_string();
        // Basic replacements
        expr = expr.replace("\\cdot", "*")
            .replace("\\times", "*")
            .replace("\\left(", "(")
            .replace("\\right)", ")")
            .replace("\\pi", "PI") // Replace \pi with PI
            .replace("\\e", "E"); // Replace \e with E
        // Implicit multiplication (e.g., 2\pi, 2e, 2(x))
        let implicit_mul_re = Regex::new(r"(\d)([a-zA-Z\(])").unwrap();
        expr = implicit_mul_re.replace_all(&expr, "$1*$2").to_string();
        let implicit_mul_paren_re = Regex::new(r"([a-zA-Z\)])(\()")
            .unwrap();
        expr = implicit_mul_paren_re.replace_all(&expr, "$1*$2").to_string();
        // LaTeX constructs
        let sqrt_re = Regex::new(r"\\sqrt\{([^}]+)\}").unwrap();
        expr = sqrt_re.replace_all(&expr, "sqrt($1)").to_string();
        let frac_re = Regex::new(r"\\frac\{([^}]+)\}\{([^}]+)\}").unwrap();
        expr = frac_re.replace_all(&expr, "($1)/($2)").to_string();
        let pow_re = Regex::new(r"\^\{([^}]+)\}").unwrap();
        expr = pow_re.replace_all(&expr, "^($1)").to_string();
        // Trigonometric & hyperbolic
        let sin_re = Regex::new(r"\\sin\{([^}]+)\}").unwrap();
        expr = sin_re.replace_all(&expr, "sin($1)").to_string();
        let cos_re = Regex::new(r"\\cos\{([^}]+)\}").unwrap();
        expr = cos_re.replace_all(&expr, "cos($1)").to_string();
        let tan_re = Regex::new(r"\\tan\{([^}]+)\}").unwrap();
        expr = tan_re.replace_all(&expr, "tan($1)").to_string();
        let sec_re = Regex::new(r"\\sec\{([^}]+)\}").unwrap();
        expr = sec_re.replace_all(&expr, "sec($1)").to_string();
        let csc_re = Regex::new(r"\\csc\{([^}]+)\}").unwrap();
        expr = csc_re.replace_all(&expr, "csc($1)").to_string();
        let cot_re = Regex::new(r"\\cot\{([^}]+)\}").unwrap();
        expr = cot_re.replace_all(&expr, "cot($1)").to_string();
        let sinh_re = Regex::new(r"\\sinh\{([^}]+)\}").unwrap();
        expr = sinh_re.replace_all(&expr, "sinh($1)").to_string();
        let cosh_re = Regex::new(r"\\cosh\{([^}]+)\}").unwrap();
        expr = cosh_re.replace_all(&expr, "cosh($1)").to_string();
        let tanh_re = Regex::new(r"\\tanh\{([^}]+)\}").unwrap();
        expr = tanh_re.replace_all(&expr, "tanh($1)").to_string();
        // Exponential
        let exp_re = Regex::new(r"\\exp\{([^}]+)\}").unwrap();
        expr = exp_re.replace_all(&expr, "exp($1)").to_string();
        // Logarithmic
        let ln_re = Regex::new(r"\\ln\{([^}]+)\}").unwrap();
        expr = ln_re.replace_all(&expr, "ln($1)").to_string();
        let log_re = Regex::new(r"\\log\{([^}]+)\}").unwrap();
        expr = log_re.replace_all(&expr, "log($1)").to_string();
        // Inverse trig
        let asin_re = Regex::new(r"\\arcsin\{([^}]+)\}").unwrap();
        expr = asin_re.replace_all(&expr, "asin($1)").to_string();
        let acos_re = Regex::new(r"\\arccos\{([^}]+)\}").unwrap();
        expr = acos_re.replace_all(&expr, "acos($1)").to_string();
        let atan_re = Regex::new(r"\\arctan\{([^}]+)\}").unwrap();
        expr = atan_re.replace_all(&expr, "atan($1)").to_string();
        // Inverse hyperbolic
        let asinh_re = Regex::new(r"\\arsinh\{([^}]+)\}").unwrap();
        expr = asinh_re.replace_all(&expr, "asinh($1)").to_string();
        let acosh_re = Regex::new(r"\\arccosh\{([^}]+)\}").unwrap();
        expr = acosh_re.replace_all(&expr, "acosh($1)").to_string();
        let atanh_re = Regex::new(r"\\arctanh\{([^}]+)\}").unwrap();
        expr = atanh_re.replace_all(&expr, "atanh($1)").to_string();
        // Limits and operators
        let lim_re = Regex::new(r"\\lim").unwrap();
        expr = lim_re.replace_all(&expr, "lim").to_string();
        let min_re = Regex::new(r"\\min\{([^}]+)\}").unwrap();
        expr = min_re.replace_all(&expr, "min($1)").to_string();
        let max_re = Regex::new(r"\\max\{([^}]+)\}").unwrap();
        expr = max_re.replace_all(&expr, "max($1)").to_string();
        let inf_re = Regex::new(r"\\inf").unwrap();
        expr = inf_re.replace_all(&expr, "inf").to_string();
        let sup_re = Regex::new(r"\\sup").unwrap();
        expr = sup_re.replace_all(&expr, "sup").to_string();
        let det_re = Regex::new(r"\\det\{([^}]+)\}").unwrap();
        expr = det_re.replace_all(&expr, "det($1)").to_string();
        let dim_re = Regex::new(r"\\dim\{([^}]+)\}").unwrap();
        expr = dim_re.replace_all(&expr, "dim($1)").to_string();
        let deg_re = Regex::new(r"\\deg\{([^}]+)\}").unwrap();
        expr = deg_re.replace_all(&expr, "deg($1)").to_string();
        let gcd_re = Regex::new(r"\\gcd\{([^}]+)\}").unwrap();
        expr = gcd_re.replace_all(&expr, "gcd($1)").to_string();
        let pr_re = Regex::new(r"\\Pr\{([^}]+)\}").unwrap();
        expr = pr_re.replace_all(&expr, "Pr($1)").to_string();
        let hom_re = Regex::new(r"\\hom\{([^}]+)\}").unwrap();
        expr = hom_re.replace_all(&expr, "hom($1)").to_string();
        let ker_re = Regex::new(r"\\ker\{([^}]+)\}").unwrap();
        expr = ker_re.replace_all(&expr, "ker($1)").to_string();
        let arg_re = Regex::new(r"\\arg\{([^}]+)\}").unwrap();
        expr = arg_re.replace_all(&expr, "arg($1)").to_string();
        // Large operators
        let sum_re = Regex::new(r"\\sum").unwrap();
        expr = sum_re.replace_all(&expr, "sum").to_string();
        let prod_re = Regex::new(r"\\prod").unwrap();
        expr = prod_re.replace_all(&expr, "prod").to_string();
        let int_re = Regex::new(r"\\int").unwrap();
        expr = int_re.replace_all(&expr, "int").to_string();
        let bigcup_re = Regex::new(r"\\bigcup").unwrap();
        expr = bigcup_re.replace_all(&expr, "bigcup").to_string();
        let bigcap_re = Regex::new(r"\\bigcap").unwrap();
        expr = bigcap_re.replace_all(&expr, "bigcap").to_string();
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
        assert_eq!(adapter.preprocess_latex(r"\sin{\pi} + \cos{0} + \tan{0}"), "sin(PI) + cos(0) + tan(0)");
        assert_eq!(adapter.preprocess_latex(r"\ln{e} + \log{10}"), "ln(e) + log(10)");
        assert_eq!(adapter.preprocess_latex(r"2\pi e"), "2*PI e");
        // Additional function tests
        assert_eq!(adapter.preprocess_latex(r"\sec{1}"), "sec(1)");
        assert_eq!(adapter.preprocess_latex(r"\csc{1}"), "csc(1)");
        assert_eq!(adapter.preprocess_latex(r"\cot{1}"), "cot(1)");
        assert_eq!(adapter.preprocess_latex(r"\sinh{0}"), "sinh(0)");
        assert_eq!(adapter.preprocess_latex(r"\cosh{0}"), "cosh(0)");
        assert_eq!(adapter.preprocess_latex(r"\tanh{0}"), "tanh(0)");
        assert_eq!(adapter.preprocess_latex(r"\lim"), "lim");
        assert_eq!(adapter.preprocess_latex(r"\min{1,2}"), "min(1,2)");
        assert_eq!(adapter.preprocess_latex(r"\max{1,2}"), "max(1,2)");
        assert_eq!(adapter.preprocess_latex(r"\inf"), "inf");
        assert_eq!(adapter.preprocess_latex(r"\sup"), "sup");
        assert_eq!(adapter.preprocess_latex(r"\det{A}"), "det(A)");
        assert_eq!(adapter.preprocess_latex(r"\dim{V}"), "dim(V)");
        assert_eq!(adapter.preprocess_latex(r"\deg{45}"), "deg(45)");
        assert_eq!(adapter.preprocess_latex(r"\gcd{8,12}"), "gcd(8,12)");
        assert_eq!(adapter.preprocess_latex(r"\Pr{A}"), "Pr(A)");
        assert_eq!(adapter.preprocess_latex(r"\hom{f}"), "hom(f)");
        assert_eq!(adapter.preprocess_latex(r"\ker{f}"), "ker(f)");
        assert_eq!(adapter.preprocess_latex(r"\arg{z}"), "arg(z)");
        assert_eq!(adapter.preprocess_latex(r"\sum"), "sum");
        assert_eq!(adapter.preprocess_latex(r"\prod"), "prod");
        assert_eq!(adapter.preprocess_latex(r"\int"), "int");
        assert_eq!(adapter.preprocess_latex(r"\bigcup"), "bigcup");
        assert_eq!(adapter.preprocess_latex(r"\bigcap"), "bigcap");
    }
}

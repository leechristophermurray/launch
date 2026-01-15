use crate::interface::ports::ICalculator;
use meval;

pub struct MevalCalculatorAdapter;

impl MevalCalculatorAdapter {
    pub fn new() -> Self {
        Self
    }
    
    fn preprocess_latex(&self, expression: &str) -> String {
        let mut expr = expression.to_string();
        // Replace common latex patterns with standard math
        // \sqrt{x} -> sqrt(x)
        // \frac{a}{b} -> (a)/(b)
        // ^ -> ^ (meval supports ^)
        // \cdot -> *
        // \times -> *
        
        expr = expr.replace("\\cdot", "*")
            .replace("\\times", "*")
            .replace("\\left(", "(")
            .replace("\\right)", ")");
            
        // Simple regex replace for sqrt
        // Note: Nested braces might fail with simple regex, but basic support:
        // \sqrt{123}
        // This is a naive implementation
        
        // Handling \frac{...}{...} is hard with regex due to nested braces.
        // For now, let's just handle simple substitutions.
        
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

// These queries are inefficient, if the aim was speed they would be done at the recipe build level.
// The reason they are not done at the build level is:
// 1: Yuck, need to handle many functions when building recipes.
// 2: Memory.
// 3: 2 is a lie, it is just nicer and easier (it might actually be better for memory but that isn't a real consideration).
// The reason it is okay to do this ASSUMES that these WILL NOT be done on a row by row level.
// PLEASE, THERE SHOULD BE NO REASON TO DO THIS ON A ROW LEVEL, USE A DIFFERENT METHOD!

use super::{Method, Recipe};

pub trait Find {
    fn contains(self, test: fn(Method) -> bool) -> bool
    // Was trying for reference; I've given up, for now.
    where
        Self: Sized,
    {
        self.get(test).is_some()
    }
    fn get(self, test: fn(Method) -> bool) -> Option<Vec<Method>>;
}

impl Find for Recipe {
    fn get(self, test: fn(Method) -> bool) -> Option<Vec<Method>> {
        match self {
            Recipe::Method(method) => method.get(test),
            _ => None, // Contains is only for methods (ingredients aren't recursive)
        }
    }
}

impl Find for Method {
    fn get(self, test: fn(Method) -> bool) -> Option<Vec<Method>> {
        if test(self.clone()) {
            Some(vec![self])
        } else {
            match self.clone() {
                Method::Value(_) => None, // Unreachable & none

                Method::BooleanCheck(_, recipe)
                | Method::UnaryOperation(_, recipe)
                | Method::Cast(_, recipe)
                | Method::Aggregate(_, recipe) => recipe.get(test),

                Method::BinaryOperation(_, left, right) => {
                    let mut results = vec![];
                    left.get(test).map(|result| results.extend(result));
                    right.get(test).map(|result| results.extend(result));
                    if results.len() > 0 {
                        Some(results)
                    } else {
                        None
                    }
                }
                Method::Function(_, arguments) => {
                    let mut results = vec![];
                    arguments
                        .into_iter()
                        .filter_map(|argument| argument.get(test))
                        .for_each(|argument| results.extend(argument));
                    if results.len() > 0 {
                        Some(results)
                    } else {
                        None
                    }
                }
                Method::Subquery(_) => None, // Maybe this needs to be expanded into, for now, fuck that.
            }
        }
    }
}

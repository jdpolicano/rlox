#[derive(Debug, Clone)]
pub enum LoxValue {
    Number(f64),
    Boolean(bool),
    Nil,
}

// impl From<Literal> for LoxValue {
//     fn from(value: Literal) -> Self {
//         match value {
//             Literal::Number { value, .. } => Self::Number(value),
//             Literal::Boolean { value, .. } => Self::Boolean(value),
//             Literal::Nil { .. } => Self::Nil,
//             _ => {
//                 println!("not implmented for '{}'", value);
//                 panic!("cannot move forward");
//             }
//         }
//     }
// }

// impl From<&Literal> for LoxValue {
//     fn from(value: &Literal) -> Self {
//         match value {
//             Literal::Number { value, .. } => Self::Number(*value),
//             Literal::Boolean { value, .. } => Self::Boolean(*value),
//             Literal::Nil { .. } => Self::Nil,
//             _ => {
//                 println!("not implmented for '{}'", value);
//                 panic!("cannot move forward");
//             }
//         }
//     }
// }

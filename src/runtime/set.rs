use crate::env::Value;
use crate::error::{LatchError, Result};
use std::collections::HashSet;

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "new" => {
            // Create new set
            Ok(Value::new_list(vec![]))
        }

        "add" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "set.add".into(), expected: 2, found: args.len() });
            }
            // Convert list to set
            let list = args[0].clone().into_list()?;
            let item = args[1].clone();
            let mut set: HashSet<String> = list.iter().map(|v| format!("{}", v)).collect();
            set.insert(format!("{}", item));
            let new_list: Vec<Value> = set.into_iter().map(Value::Str).collect();
            Ok(Value::new_list(new_list))
        }

        "remove" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "set.remove".into(), expected: 2, found: args.len() });
            }
            let list = args[0].clone().into_list()?;
            let item = args[1].clone();
            let mut set: HashSet<String> = list.iter().map(|v| format!("{}", v)).collect();
            set.remove(&format!("{}", item));
            let new_list: Vec<Value> = set.into_iter().map(Value::Str).collect();
            Ok(Value::new_list(new_list))
        }

        "has" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "set.has".into(), expected: 2, found: args.len() });
            }
            let list = args[0].clone().into_list()?;
            let item = format!("{}", args[1]);
            let set: HashSet<String> = list.iter().map(|v| format!("{}", v)).collect();
            Ok(Value::Bool(set.contains(&item)))
        }

        "union" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "set.union".into(), expected: 2, found: args.len() });
            }
            let list1 = args[0].clone().into_list()?;
            let list2 = args[1].clone().into_list()?;
            let mut set: HashSet<String> = list1.iter().map(|v| format!("{}", v)).collect();
            set.extend(list2.iter().map(|v| format!("{}", v)));
            let new_list: Vec<Value> = set.into_iter().map(Value::Str).collect();
            Ok(Value::new_list(new_list))
        }

        "intersection" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "set.intersection".into(), expected: 2, found: args.len() });
            }
            let list1 = args[0].clone().into_list()?;
            let list2 = args[1].clone().into_list()?;
            let set1: HashSet<String> = list1.iter().map(|v| format!("{}", v)).collect();
            let set2: HashSet<String> = list2.iter().map(|v| format!("{}", v)).collect();
            let result: HashSet<String> = set1.intersection(&set2).cloned().collect();
            let new_list: Vec<Value> = result.into_iter().map(Value::Str).collect();
            Ok(Value::new_list(new_list))
        }

        "difference" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "set.difference".into(), expected: 2, found: args.len() });
            }
            let list1 = args[0].clone().into_list()?;
            let list2 = args[1].clone().into_list()?;
            let set1: HashSet<String> = list1.iter().map(|v| format!("{}", v)).collect();
            let set2: HashSet<String> = list2.iter().map(|v| format!("{}", v)).collect();
            let result: HashSet<String> = set1.difference(&set2).cloned().collect();
            let new_list: Vec<Value> = result.into_iter().map(Value::Str).collect();
            Ok(Value::new_list(new_list))
        }

        _ => Err(LatchError::UnknownMethod { module: "set".into(), method: method.into() }),
    }
}

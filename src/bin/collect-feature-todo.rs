use linked_sql::feature_todo::FeatureTodo;

fn main() {
    println!("Feature TODO Inventory:");
    println!("======================\n");

    let todos: Vec<_> = inventory::iter::<FeatureTodo>().collect();
    
    if todos.is_empty() {
        println!("No feature TODOs found.");
        return;
    }

    let mut by_feature: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    
    for todo in &todos {
        by_feature.entry(todo.feature)
            .or_insert_with(Vec::new)
            .push(todo.comment);
    }

    for (feature, comments) in by_feature.iter() {
        println!("Feature: {}", feature);
        for comment in comments {
            println!("  - {}", comment);
        }
        println!();
    }

    println!("Total: {} TODO items across {} features", todos.len(), by_feature.len());
}

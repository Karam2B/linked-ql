#!/usr/bin/env python3
"""
Add inventory submissions to all cfg-gated modules
"""
import re
import sys

# Module to comment mapping
modules = [
    ("src/operations/fetch_one.rs", "impl_operation_for_fetch_one", "impl OperationOutput + Operation<S> for FetchOne"),
    ("src/operations/fetch_one_v2.rs", "impl_operation_for_fetch_one", "impl OperationOutput + Operation<S> for FetchOne v2"),
    ("src/operations/fetch_many.rs", "impl_expressions_for_operation_for_empty", "impl ExpressionsForOperation for Empty"),
    ("src/operations/fetch_many.rs", "impl_operation_output_for_fetch_many", "impl OperationOutput for FetchMany"),
    ("src/operations/fetch_many.rs", "impl_first_item_trait_for_tuples", "impl FirstItemTrait for tuples"),
    ("src/operations/fetch_many.rs", "impl_operation_for_fetch_many", "impl Operation<S> for FetchMany"),
    ("src/operations/junction.rs", "impl_operation_for_insert_junction_and_fetch", "impl OperationOutput + Operation<S> for InsertJunctionAndFetch"),
    ("src/links/utils.rs", None, "ConventionalForeignKeyName and related utils"),
    ("src/links/mod.rs", "impl_expressions_for_operation_for_many_links_tuple", "impl ExpressionsForOperation for ManyLinks<(L0, L1)>"),
    ("src/links/mod.rs", "impl_expressions_for_operation_for_many_links_vec", "impl ExpressionsForOperation for ManyLinks<Vec<T>>"),
    ("src/links/v1_insert_one.rs", "impl_operation_for_insert_one", "impl OperationOutput + Operation<S> for InsertOne (v1)"),
    ("src/links/relation_one_to_many.rs", "impl_on_migrate", "impl MigrateExpression for OneToMany"),
    ("src/links/relation_one_to_many.rs", "impl_link_fetch_many", "impl LinkFetch for OneToMany"),
    ("src/links/relation_one_to_many.rs", "impl_set_new_for_insert", "impl InsertOneLink for SetNew<OneToMany>"),
    ("src/links/relation_one_to_many.rs", "impl_set_id_for_insert", "impl InsertOneLink for SetId<OneToMany>"),
    ("src/links/relation_one_to_many.rs", "impl_set_id_for_update", "impl UpdateLink for SetId<OneToMany>"),
    ("src/links/relation_one_to_many.rs", "impl_set_new_for_update", "impl UpdateLink for SetNew<OneToMany>"),
    ("src/links/relation_one_to_many.rs", "impl_for_delete", "impl DeleteLink for OneToMany"),
    ("src/links/relation_one_to_many_inverse.rs", "impl_link_fetch", "impl LinkFetch for OneToManyInverse"),
    ("src/links/relation_many_to_many.rs", "impl_insert_junction_many_rows", "impl InsertJunctionManyRows for ManyToMany"),
    ("src/links/relation_many_to_many.rs", "impl_on_migrate", "impl MigrateExpression for ManyToMany"),
    ("src/links/relation_many_to_many.rs", "impl_link_fetch_many", "impl LinkFetch for ManyToMany"),
    ("src/links/relation_many_to_many.rs", "impl_many_to_many_set_new", "impl InsertOneLink for SetNew<ManyToMany>"),
    ("src/links/relation_many_to_many.rs", "impl_set_id_for_insert", "impl InsertOneLink for SetJunctionId<ManyToMany>"),
    ("src/links/relation_many_to_many.rs", "impl_set_junzcction_id_for_update", "impl UpdateLink for SetJunctionId<ManyToMany>"),
    ("src/links/relation_many_to_many.rs", "impl_remove_junction_id_for_update", "impl UpdateLink for RemoveJunctionId<ManyToMany>"),
    ("src/links/relation_many_to_many.rs", "impl_for_delete", "impl DeleteLink for ManyToMany"),
    ("src/links/timestamp.rs", "impl_on_migrate", "impl MigrateExpression for Timestamp"),
    ("src/links/timestamp.rs", "impl_on_insert", "impl OnInsert for Timestamp"),
    ("src/links/junction_table_op.rs", "impl_one_insert", "impl Operation for InsertJunction"),
    ("src/links/junction_table_op.rs", "impl_on_delete", "impl Operation for DeleteJunction"),
    ("src/links/fetch_linked_records.rs", "impl_many_to_many_junction_names", "impl JunctionNames for ManyToMany"),
    ("src/links/fetch_linked_records.rs", "impl_insert_junction_row", "impl InsertJunctionRow"),
    ("src/links/fetch_linked_records.rs", "impl_delete_junction_row", "impl DeleteJunctionRow"),
    ("src/links/fetch_linked_records.rs", "impl_insert_junction_and_fetch", "impl InsertJunctionAndFetch"),
    ("src/links/fetch_linked_records.rs", "impl_select_junction_to_ids", "impl SelectJunctionToIds"),
    ("src/links/fetch_linked_records.rs", "impl_fetch_one_to_many_inverse_linked", "impl FetchOneToManyInverseLinked"),
    ("src/links/fetch_linked_records.rs", "impl_fetch_many_to_many_linked", "impl FetchManyToManyLinked"),
    ("src/json_client/dynamic_collection.rs", "common_expression_impls", "common expression impls for DynamicCollection"),
    ("src/json_client/dynamic_collection.rs", "arc_collection_impls", "Arc<DynamicCollection> impls"),
    ("src/json_client/dynamic_collection.rs", "impl_on_migrate", "impl MigrateExpression for DynamicCollection"),
    ("src/json_client/compat.rs", "dynamic_input", "DynamicInsertInput compat"),
    ("src/json_client/compat.rs", "insert_sets", "insert sets trait objects"),
    ("src/json_client/op_insert_one_trait_extension.rs", "impl_json_insert_one_link", "JsonInsertOneLink trait impls"),
    ("src/json_client/op_update_one_trait_extension.rs", "impl_json_update_one_link", "JsonUpdateOneLink trait impls"),
    ("src/json_client/op_fetch_many_trait_extension.rs", "impl_json_link_fetch_many", "JsonLinkFetchMany trait impls"),
    ("src/json_client/select_items_trait_object.rs", "impl_select_items_trait_object", "SelectItemsTraitObject impls"),
    ("src/json_client/ops.rs", None, "ops! macro invocation"),
    ("src/test_module.rs", None, "test module macro-generated crossover impls"),
]

def add_inventory_to_file(filepath, module_name, comment):
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        if "inventory::submit!" in content and comment in content:
            print(f"✓ {filepath} ({module_name or 'top-level'}): already has inventory")
            return
        
        if module_name:
            # Find the module and add inventory after use statements
            pattern = rf"(#\[cfg\(not\(feature = \"in_dev_op2\"\)\)\]\s+mod {module_name} \{{[^\}}]*?use[^;]+;)(\s+)"
            
            inventory_block = f'''

    #[cfg(feature = "inventory")]
    inventory::submit! {{
        crate::feature_todo::FeatureTodo {{
            feature: "in_dev_op2",
            comment: "{filepath}: {comment}",
        }}
    }}'''
            
            new_content = re.sub(pattern, rf'\1{inventory_block}\2', content, count=1)
        else:
            # For top-level cfg gates, add after the cfg attribute
            pattern = r'(#\[cfg\(not\(feature = "in_dev_op2"\)\)\]\s+)'
            
            inventory_block = f'''#[cfg(feature = "inventory")]
inventory::submit! {{
    crate::feature_todo::FeatureTodo {{
        feature: "in_dev_op2",
        comment: "{filepath}: {comment}",
    }}
}}

'''
            
            # Only add if not already present
            if "inventory::submit!" not in content:
                new_content = re.sub(pattern, rf'{inventory_block}\1', content, count=1, flags=re.MULTILINE)
            else:
                new_content = content
        
        if new_content != content:
            with open(filepath, 'w') as f:
                f.write(new_content)
            print(f"✓ {filepath} ({module_name or 'top-level'}): added inventory")
        else:
            print(f"⚠ {filepath} ({module_name or 'top-level'}): no change (pattern not found)")
            
    except Exception as e:
        print(f"✗ {filepath}: {e}")

if __name__ == "__main__":
    for filepath, module_name, comment in modules:
        add_inventory_to_file(filepath, module_name, comment)

#!/usr/bin/env python3
"""
Move inventory submissions outside cfg gates
"""
import re
import os

def process_file(filepath):
    if not os.path.exists(filepath):
        return False
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Pattern: find cfg gate followed by module with inventory inside
    pattern = r'(#\[cfg\(not\(feature = "in_dev_op2"\)\)\]\s*(?:pub(?:\(crate\))?\s+)?mod\s+\w+\s*\{[^\{]*?use[^\}]+?)\s+(#\[cfg\(feature = "inventory"\)\]\s+inventory::submit!\s*\{[^\}]+\}[^\}]+\})'
    
    # Move inventory submission before cfg gate
    def replacer(match):
        cfg_and_mod = match.group(1)
        inventory = match.group(2)
        # Find the cfg line
        cfg_match = re.search(r'(#\[cfg\(not\(feature = "in_dev_op2"\)\)\])', cfg_and_mod)
        if cfg_match:
            before_cfg = cfg_and_mod[:cfg_match.start()]
            cfg_line = cfg_match.group(1)
            after_cfg = cfg_and_mod[cfg_match.end():]
            return f'{before_cfg}{inventory}\n\n{cfg_line}{after_cfg}'
        return match.group(0)
    
    new_content = re.sub(pattern, replacer, content, flags=re.DOTALL)
    
    # Also handle top-level cfg gates (not in modules)
    pattern2 = r'(#\[cfg\(not\(feature = "in_dev_op2"\)\)\]\s*)(\n#\[cfg\(feature = "inventory"\)\]\s+inventory::submit!\s*\{[^\}]+\}[^\}]+\}\s*\n)'
    
    def replacer2(match):
        cfg = match.group(1)
        inventory = match.group(2).strip()
        return f'{inventory}\n\n{cfg}'
    
    new_content = re.sub(pattern2, replacer2, new_content)
    
    if new_content != content:
        with open(filepath, 'w') as f:
            f.write(new_content)
        return True
    return False

files_to_process = [
    "src/operations/delete.rs",
    "src/operations/update.rs",
    "src/operations/insert.rs",
    "src/operations/fetch_one.rs",
    "src/operations/fetch_one_v2.rs",
    "src/operations/fetch_many.rs",
    "src/operations/junction.rs",
    "src/links/utils.rs",
    "src/links/mod.rs",
    "src/links/v1_insert_one.rs",
    "src/links/relation_one_to_many.rs",
    "src/links/relation_one_to_many_inverse.rs",
    "src/links/relation_many_to_many.rs",
    "src/links/timestamp.rs",
    "src/links/junction_table_op.rs",
    "src/links/fetch_linked_records.rs",
    "src/json_client/dynamic_collection.rs",
    "src/json_client/compat.rs",
    "src/json_client/op_insert_one_trait_extension.rs",
    "src/json_client/op_update_one_trait_extension.rs",
    "src/json_client/op_fetch_many_trait_extension.rs",
    "src/json_client/select_items_trait_object.rs",
    "src/json_client/ops.rs",
    "src/test_module.rs",
]

changed = []
for filepath in files_to_process:
    if process_file(filepath):
        changed.append(filepath)
        print(f"✓ {filepath}")
    else:
        print(f"⚠ {filepath}: no change")

print(f"\nChanged {len(changed)} files")

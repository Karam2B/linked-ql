#!/usr/bin/env python3
"""
Find all inventory submissions inside cfg gates and move them out
"""
import re
import os
import glob

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # Pattern 1: inventory inside a module (after use statements)
    # Match: mod name { ... use ...; ... inventory::submit! { ... } }
    pattern = r'(#\[cfg\(not\(feature = "in_dev_op2"\)\)\]\s+(?:pub(?:\(crate\))?\s+)?mod\s+(\w+)\s*\{[^}]*?use[^}]+?)\s+(#\[cfg\(feature = "inventory"\)\]\s+inventory::submit!\s*\{[^}]+?\}[^}]+?\})'
    
    def move_out(match):
        full_match = match.group(0)
        mod_start = match.group(1)
        inventory_block = match.group(3)
        
        # Find the cfg line at the start
        cfg_match = re.search(r'(#\[cfg\(not\(feature = "in_dev_op2"\)\)\])', mod_start)
        if not cfg_match:
            return full_match
        
        cfg_start = cfg_match.start()
        before_cfg = mod_start[:cfg_start]
        cfg_line = cfg_match.group(1)
        after_cfg = mod_start[cfg_match.end():]
        
        # Return with inventory before cfg
        return f'{before_cfg}{inventory_block}\n\n{cfg_line}{after_cfg}'
    
    content = re.sub(pattern, move_out, content, flags=re.DOTALL)
    
    # Pattern 2: More complex nested case
    # This handles cases where there might be more code between use and inventory
    lines = content.split('\n')
    result = []
    i = 0
    pending_inventory = []
    
    while i < len(lines):
        line = lines[i]
        
        # Check if we're starting a cfg-gated module
        if '#[cfg(not(feature = "in_dev_op2"))]' in line:
            # Look ahead for inventory inside this module
            mod_start_idx = i
            brace_count = 0
            found_inventory = False
            inventory_lines = []
            inventory_start = -1
            inventory_end = -1
            
            # Find the module block
            j = i + 1
            while j < len(lines):
                if 'mod ' in lines[j] and '{' in lines[j]:
                    brace_count = 1
                    j += 1
                    break
                j += 1
            
            # Scan inside the module for inventory
            while j < len(lines) and brace_count > 0:
                if '{' in lines[j]:
                    brace_count += lines[j].count('{')
                if '}' in lines[j]:
                    brace_count -= lines[j].count('}')
                
                if '#[cfg(feature = "inventory")]' in lines[j]:
                    inventory_start = j
                    # Collect the inventory block
                    while j < len(lines):
                        inventory_lines.append(lines[j])
                        if 'inventory::submit!' in lines[j]:
                            # Find the end of the submit block
                            depth = 0
                            while j < len(lines):
                                for char in lines[j]:
                                    if char == '{': depth += 1
                                    if char == '}': depth -= 1
                                inventory_lines.append(lines[j])
                                j += 1
                                if depth == 0:
                                    inventory_end = j
                                    found_inventory = True
                                    break
                            break
                        j += 1
                    break
                j += 1
            
            if found_inventory:
                # Output inventory before cfg
                result.extend(inventory_lines)
                result.append('')
                
                # Output cfg line
                result.append(line)
                
                # Output module lines, skipping the inventory we already moved
                for k in range(i + 1, len(lines)):
                    if k < inventory_start or k >= inventory_end:
                        result.append(lines[k])
                        if k >= j:  # We've processed the whole module
                            i = k + 1
                            break
                continue
        
        result.append(line)
        i += 1
    
    content = '\n'.join(result)
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

# Find all Rust files
rust_files = []
for pattern in ['src/**/*.rs', 'src/*.rs']:
    rust_files.extend(glob.glob(pattern, recursive=True))

changed = []
for filepath in rust_files:
    if fix_file(filepath):
        changed.append(filepath)
        print(f"✓ {filepath}")

if not changed:
    print("No files needed changes")
else:
    print(f"\nFixed {len(changed)} files")

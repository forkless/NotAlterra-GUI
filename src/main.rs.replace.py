import re
f = open('src/main.rs')
text = f.read()
f.close()

# List of (old_snippet, new_snippet) — both must match exactly
replacements = [
    ('guard::log_action("MANUAL_BAK", &save_folder.display().to_string()',
     'guard::log_action("MANUAL_BAK", &guard::sanitize_path(&save_folder.display().to_string())'),
    ('guard::log_action("AUTO_BAK", &format!("pre-restore → {}", save_folder.display())',
     'guard::log_action("AUTO_BAK", &format!("pre-restore → {}", guard::sanitize_path(&save_folder.display().to_string()))'),
    ('guard::log_action("RESTORE", &format!("{} → {}", name, save_folder.display())',
     'guard::log_action("RESTORE", &format!("{} → {}", name, guard::sanitize_path(&save_folder.display().to_string()))'),
    ('guard::log_action("CONFIG_BAK", &result.dest_dir.display().to_string()',
     'guard::log_action("CONFIG_BAK", &guard::sanitize_path(&result.dest_dir.display().to_string())'),
    ('guard::log_action("AUTO_BAK", &format!("ini pre-restore → {}", ini_path.display())',
     'guard::log_action("AUTO_BAK", &format!("ini pre-restore → {}", guard::sanitize_path(&ini_path.display().to_string()))'),
    ('guard::log_action("CONFIG_RESTORE", &chosen.display().to_string()',
     'guard::log_action("CONFIG_RESTORE", &guard::sanitize_path(&chosen.display().to_string())'),
    ('guard::log_action("CONFIG_DEL", &ini_path.display().to_string()',
     'guard::log_action("CONFIG_DEL", &guard::sanitize_path(&ini_path.display().to_string())'),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        print(f'WARN: found {count} occurences of: {old[:50]}...')
    text = text.replace(old, new)

open('src/main.rs', 'w').write(text)
print('done')

#!/usr/bin/env fish

# Настройка цветов
set -l green (set_color green)
set -l red (set_color red)
set -l yellow (set_color yellow)
set -l blue (set_color cyan)
set -l normal (set_color normal)

echo "🛠️  $yellow Запускаем Full CI Pipeline (Clippy + Tests + Examples)...$normal"
echo "------------------------------------------------------"

set -l script_dir (cd (dirname (status filename)); and pwd)

# Ищем все директории с Cargo.toml (исключаем target)
set -l projects (find "$script_dir" -name "Cargo.toml" -not -path "*/target/*")

set -l failed_clippy
set -l failed_tests

for project in $projects
    set -l project_dir (dirname "$project")
    set -l project_name (basename $project_dir)

    echo "📂 $blue Проект:$normal $project_name ($project_dir)"
    if not pushd "$project_dir" >/dev/null
        echo "  ❌ $red Не удалось войти в директорию: $project_dir$normal"
        set -a failed_clippy $project_dir
        set -a failed_tests $project_dir
        echo "------------------------------------------------------"
        continue
    end

    if string match -q "*/ruby/ext/aam_rs" "$project_dir"
        if not type -q ruby
            echo "  ⚠️  $yellow Ruby не найден, пропускаем optional-проект.$normal"
            popd >/dev/null
            echo "------------------------------------------------------"
            continue
        end
    end

    echo "  🔍 $yellow Запуск Clippy...$normal"
    set -l clippy_cmd cargo clippy --all-targets --all-features
    if test "$project_dir" != "$script_dir"
        set clippy_cmd $clippy_cmd -- -D warnings
    end

    if not $clippy_cmd
        set -a failed_clippy $project_dir
        echo "  ❌ $red Clippy Error!$normal"
    else
        echo "  ✅ $green Clippy OK$normal"
    end

    echo "  🧪 $yellow Запуск Tests...$normal"
    if not cargo test --all-features
        set -a failed_tests $project_dir
        echo "  ❌ $red Tests Failed!$normal"
    else
        echo "  ✅ $green Tests PASSED$normal"
    end

    echo "  📚 $yellow Проверка Examples...$normal"
    if not cargo check --examples
        echo "  ❌ $red Examples broken!$normal"
        if not contains $project_dir $failed_clippy
             set -a failed_clippy "$project_dir (examples)"
        end
    else
        echo "  ✅ $green Examples OK$normal"
    end

    popd >/dev/null
    echo "------------------------------------------------------"
end

# Финальный отчет
echo "📊 $blue ИТОГИ ПРОВЕРКИ:$normal"

if test (count $failed_clippy) -eq 0 -a (count $failed_tests) -eq 0
    echo "🚀 $green Все проверки пройдены! Код — лютый флекс.$normal"
    exit 0
else
    if test (count $failed_clippy) -gt 0
        echo "💀 $red Проблемы с Clippy/Компиляцией:$normal"
        for f in $failed_clippy; echo "  - $f"; end
    end
    if test (count $failed_tests) -gt 0
        echo "🧨 $red Заваленные тесты:$normal"
        for f in $failed_tests; echo "  - $f"; end
    end
    exit 1
end
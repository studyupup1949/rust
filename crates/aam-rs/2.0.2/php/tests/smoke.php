<?php

declare(strict_types=1);

require_once __DIR__ . '/../src/AamPhp.php';

use RustGames\Aam\AamDocument;

if (!class_exists('FFI')) {
    fwrite(STDOUT, "PHP FFI extension is not enabled; skipping smoke test" . PHP_EOL);
    exit(0);
}

$aam = new AamDocument("host = localhost\nport = 8080");
$value = $aam->get('host');

if ($value !== 'localhost') {
    fwrite(STDERR, "Expected localhost, got: " . var_export($value, true) . PHP_EOL);
    exit(1);
}

echo "PHP smoke test passed" . PHP_EOL;



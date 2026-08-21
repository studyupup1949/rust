<?php

declare(strict_types=1);

namespace RustGames\Aam;

use RuntimeException;
use FFI;
use FFI\CData;

/**
 * High-performance PHP FFI wrapper for the AAM Rust parser.
 */
final class AamDocument
{
    private FFI $ffi;
    private ?CData $handle = null;

    public function __construct(string $content, ?string $libPath = null)
    {
        if (!class_exists('FFI')) {
            throw new RuntimeException('PHP FFI extension is not enabled. Вруби ffi.enable=true в php.ini!');
        }

        $lib = $libPath ?? getenv('AAM_RS_LIB') ?: __DIR__ . '/../../target/release/libaam_rs.so';

        $this->ffi = FFI::cdef(<<<'CDEF'
            typedef struct AamHandle AamHandle;
            AamHandle* aam_new(void);
            void aam_free(AamHandle* handle);
            int aam_parse(AamHandle* handle, const char* content);
            int aam_load(AamHandle* handle, const char* path);
            char* aam_format(AamHandle* handle, const char* content);

            char* aam_get(AamHandle* handle, const char* key);
            char* aam_find(AamHandle* handle, const char* query);
            char* aam_deep_search(AamHandle* handle, const char* pattern);
            char* aam_reverse_search(AamHandle* handle, const char* value);

            char* aam_schema_names(AamHandle* handle);
            char* aam_type_names(AamHandle* handle);

            void aam_string_free(char* s);
            const char* aam_last_error(AamHandle* handle);
        CDEF, $lib);

        $this->handle = $this->ffi->aam_new();
        if (FFI::isNull($this->handle)) {
            throw new RuntimeException('Failed to allocate AAM handle');
        }

        $rc = $this->ffi->aam_parse($this->handle, $content);
        if ($rc !== 0) {
            $this->throwLastError('Native parse failed');
        }
    }

    public function __destruct()
    {
        if ($this->handle !== null && !FFI::isNull($this->handle)) {
            $this->ffi->aam_free($this->handle);
            $this->handle = null;
        }
    }

    public function reload(string $content): void
    {
        $this->ffi->aam_reload($this->handle, $content);
    }

    public function format(string $content): string
    {
        $formattedPtr = $this->ffi->aam_format($this->handle, $content);
        if ($formattedPtr === null) {
            $this->throwLastError('Native format failed');
        }

        try {
            return FFI::string($formattedPtr);
        } finally {
            $this->ffi->aam_string_free($formattedPtr);
        }
    }

    public function get(string $key): ?string
    {
        $valuePtr = $this->ffi->aam_get($this->handle, $key);
        if ($valuePtr === null) {
            return null;
        }

        try {
            return FFI::string($valuePtr);
        } finally {
            $this->ffi->aam_string_free($valuePtr);
        }
    }

    public function reverseSearch(string $value): array
    {
        $ptr = $this->ffi->aam_reverse_search($this->handle, $value);
        return $this->parseCList($ptr);
    }

    public function find(string $query): array
    {
        $ptr = $this->ffi->aam_find($this->handle, $query);
        return $this->parseCMap($ptr);
    }

    public function deepSearch(string $pattern): array
    {
        $ptr = $this->ffi->aam_deep_search($this->handle, $pattern);
        return $this->parseCMap($ptr);
    }

    public function schemaNames(): array
    {
        $ptr = $this->ffi->aam_schema_names($this->handle);
        return $this->parseCList($ptr);
    }

    public function typeNames(): array
    {
        $ptr = $this->ffi->aam_type_names($this->handle);
        return $this->parseCList($ptr);
    }

    private function throwLastError(string $defaultMsg): void
    {
        $err = $this->ffi->aam_last_error($this->handle);
        $msg = $err !== null ? FFI::string($err) : $defaultMsg;
        throw new RuntimeException($msg);
    }

    private function parseCList(?CData $ptr): array
    {
        if ($ptr === null) {
            return [];
        }
        try {
            $str = FFI::string($ptr);
            return $str === '' ? [] : explode(',', $str);
        } finally {
            $this->ffi->aam_string_free($ptr);
        }
    }

    private function parseCMap(?CData $ptr): array
    {
        if ($ptr === null) {
            return [];
        }
        try {
            $str = FFI::string($ptr);
            if ($str === '') {
                return [];
            }
            $result = [];
            foreach (explode("\n", $str) as $line) {
                $parts = explode('=', $line, 2);
                if (count($parts) === 2) {
                    $result[$parts[0]] = $parts[1];
                }
            }
            return $result;
        } finally {
            $this->ffi->aam_string_free($ptr);
        }
    }
}

final class SchemaField
{
    private function __construct(
        public readonly string $name,
        public readonly string $typeName,
        public readonly bool $optional,
    ) {
    }

    public static function required(string $name, string $typeName): self
    {
        return new self($name, $typeName, false);
    }

    public static function optional(string $name, string $typeName): self
    {
        return new self($name, $typeName, true);
    }

    public function toAam(): string
    {
        return $this->optional ? "{$this->name}*: {$this->typeName}" : "{$this->name}: {$this->typeName}";
    }
}

final class AamBuilder
{
    /** @var list<string> */
    private array $lines = [];

    public function addLine(string $key, string $value): self
    {
        $this->lines[] = "$key = $value";
        return $this;
    }

    public function comment(string $text): self
    {
        $this->lines[] = "# $text";
        return $this;
    }

    /** @param list<SchemaField> $fields */
    public function schema(string $name, array $fields): self
    {
        $rendered = array_map(static fn (SchemaField $field): string => $field->toAam(), $fields);
        $this->lines[] = '@schema ' . $name . ' { ' . implode(', ', $rendered) . ' }';
        return $this;
    }

    /** @param list<string> $schemas */
    public function derive(string $path, array $schemas = []): self
    {
        $suffix = $schemas === [] ? '' : '::' . implode('::', $schemas);
        $this->lines[] = '@derive ' . $path . $suffix;
        return $this;
    }

    public function import(string $path): self
    {
        $this->lines[] = '@import ' . $path;
        return $this;
    }

    public function typeAlias(string $alias, string $typeName): self
    {
        $this->lines[] = '@type ' . $alias . ' = ' . $typeName;
        return $this;
    }

    public function build(): string
    {
        return implode("\n", $this->lines);
    }

    public function toFile(string $path): void
    {
        file_put_contents($path, $this->build());
    }
}

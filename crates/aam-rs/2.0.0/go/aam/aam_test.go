package aam_test

import (
	"testing"

	"github.com/INiNiDS/aam-rs/go/aam"
)

// ── Construction ─────────────────────────────────────────────────────────────

func TestNew(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatalf("New() unexpected error: %v", err)
	}
	doc.Close()
}

func TestParse_Basic(t *testing.T) {
	doc, err := aam.Parse("host = localhost\nport = 8080\n")
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}
	defer doc.Close()

	assertGet(t, doc, "host", "localhost")
	assertGet(t, doc, "port", "8080")
}

func TestParse_MultiLine(t *testing.T) {
	content := "name = Alice\nrole = admin\nlang = go\n"
	doc, err := aam.Parse(content)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}
	defer doc.Close()

	assertGet(t, doc, "name", "Alice")
	assertGet(t, doc, "role", "admin")
	assertGet(t, doc, "lang", "go")
}

func TestLoad_NonExistentFile(t *testing.T) {
	_, err := aam.Load("/tmp/aam_test_nonexistent_file_abc123.aam")
	if err == nil {
		t.Fatal("Load() of non-existent file: expected error, got nil")
	}
}

// ── Get ───────────────────────────────────────────────────────────────────────

func TestGet_NotFound(t *testing.T) {
	doc, err := aam.Parse("x = 1\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	if _, ok := doc.Get("missing_key"); ok {
		t.Error("Get(missing_key): want false, got true")
	}
}

func TestGet_ClosedHandle(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	doc.Close()

	if _, ok := doc.Get("anything"); ok {
		t.Error("Get on closed handle: want false, got true")
	}
}

// ── Find / ReverseSearch / DeepSearch ────────────────────────────────────────

func TestFind_ByKeyAndValue(t *testing.T) {
	doc, err := aam.Parse("username = alice\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	byKey := doc.Find("username")
	if byKey["username"] != "alice" {
		t.Fatalf("Find(username) missing expected pair, got: %#v", byKey)
	}

	byValue := doc.Find("alice")
	if byValue["username"] != "alice" {
		t.Fatalf("Find(alice) missing expected pair, got: %#v", byValue)
	}
}

func TestReverseSearch_Found(t *testing.T) {
	doc, err := aam.Parse("username = alice\nrole = admin\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	keys := doc.ReverseSearch("alice")
	if len(keys) == 0 {
		t.Fatalf("ReverseSearch(alice) returned no keys")
	}
	if keys[0] != "username" {
		t.Fatalf("ReverseSearch(alice)[0] = %q; want username", keys[0])
	}
}

func TestDeepSearch_ByPattern(t *testing.T) {
	doc, err := aam.Parse("app_host = localhost\ndb_host = db\nmode = prod\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	res := doc.DeepSearch("host")
	if len(res) != 2 {
		t.Fatalf("DeepSearch(host) len = %d; want 2 (got %#v)", len(res), res)
	}
	if res["app_host"] != "localhost" || res["db_host"] != "db" {
		t.Fatalf("DeepSearch(host) unexpected map: %#v", res)
	}
}

func TestFind_ClosedHandle_ReturnsNil(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	doc.Close()

	if got := doc.Find("anything"); got != nil {
		t.Fatalf("Find on closed handle: want nil, got %#v", got)
	}
}

// ── Formatting ───────────────────────────────────────────────────────────────

func TestFormat_ReturnsOutput(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	formatted, err := doc.Format("host=localhost\n")
	if err != nil {
		t.Fatalf("Format() unexpected error: %v", err)
	}
	if formatted == "" {
		t.Fatal("Format() returned empty output")
	}
}

func TestFormat_ClosedHandle_ReturnsError(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	doc.Close()

	if _, err := doc.Format("x = 1\n"); err == nil {
		t.Fatal("Format on closed handle: expected error, got nil")
	}
}

// ── Metadata ─────────────────────────────────────────────────────────────────

func TestSchemaNamesAndTypeNames_NoPanic(t *testing.T) {
	doc, err := aam.Parse("host = localhost\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	_ = doc.SchemaNames()
	_ = doc.TypeNames()
	// these may be empty depending on input; this test only verifies API wiring.

	if doc.SchemaNames() == nil {
		t.Fatal("SchemaNames() returned nil on open handle")
	}
	if doc.TypeNames() == nil {
		t.Fatal("TypeNames() returned nil on open handle")
	}
}

// ── Close / finalizer ────────────────────────────────────────────────────────

func TestClose_Idempotent(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	// Close twice — must not panic or double-free.
	doc.Close()
	doc.Close()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

func assertGet(t *testing.T, doc *aam.AAM, key, want string) {
	t.Helper()
	got, ok := doc.Get(key)
	if !ok {
		t.Errorf("Get(%q): not found (want %q)", key, want)
		return
	}
	if got != want {
		t.Errorf("Get(%q) = %q; want %q", key, got, want)
	}
}


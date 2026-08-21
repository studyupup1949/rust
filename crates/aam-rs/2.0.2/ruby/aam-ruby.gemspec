# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name = 'aam-ruby'
  spec.version = '2.0.2' # x-release-please-version
  spec.authors = ['INiNiDS']
  spec.email = ['ininids@ininids.in.rs']

  spec.summary = 'Ruby bindings for aam-rs AAML parser'
  spec.description = 'Ruby bindings for the aam-rs AAML parser built with Rust and magnus.'
  spec.homepage = 'https://github.com/ininids/aam-rs'
  spec.license = 'MIT'
  spec.required_ruby_version = '>= 3.1'

  spec.metadata = {
    'homepage_uri' => spec.homepage,
    'source_code_uri' => spec.homepage,
    'changelog_uri' => 'https://github.com/ininids/aam-rs/blob/main/CHANGELOG.md'
  }

  spec.files = Dir.chdir(__dir__) do
    Dir[
      'README.md',
      'LICENSE-MIT',
      'LICENSE-APACHE',
      'CREDITS.html',
      'lib/**/*.rb',
      'lib/aam_rs_ruby.*',
      'tests/**/*.rb',
      'ext/aam_rs/target/release/libaam_rs_ruby.*'
    ]
  end

  spec.require_paths = ['lib']
end


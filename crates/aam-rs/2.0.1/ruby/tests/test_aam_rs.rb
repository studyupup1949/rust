# frozen_string_literal: true

require 'minitest/autorun'
require_relative '../lib/aam_rb'

class AamRsTest < Minitest::Test
  def test_parse_and_get
    doc = AamRb::AAM.parse("host = localhost\nport = 8080")
    assert_equal 'localhost', doc.get('host')
  end

  def test_find_returns_key_value_pairs
    doc = AamRb::AAM.parse("host = localhost\nport = 8080")
    matches = doc.find('host')
    assert_includes matches, ['host', 'localhost']
  end

  def test_reverse_search_returns_matching_keys
    doc = AamRb::AAM.parse("host = localhost\nalias = localhost")
    keys = doc.reverse_search('localhost')
    assert_includes keys, 'host'
    assert_includes keys, 'alias'
  end

  def test_get_returns_nil_for_unknown_key
    doc = AamRb::AAM.parse("host = localhost")
    assert_nil doc.get('missing')
  end

  def test_parse_raises_on_invalid_content
    assert_raises(RuntimeError) do
      AamRb::AAM.parse('invalid_line_without_equals')
    end
  end
end


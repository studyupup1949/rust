# frozen_string_literal: true

begin
  require_relative 'aam__ruby'
rescue LoadError
  begin
    require_relative '../ext/aam_rs/target/release/libaam_ruby'
  rescue LoadError
    require_relative '../ext/aam_rs/target/debug/libaam_ruby'
  end
end


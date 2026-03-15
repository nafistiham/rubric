begin
  foo
rescue StandardError => e
  bar
end

begin
  baz
rescue ArgumentError, TypeError => e
  qux
end

begin
  something
rescue RuntimeError
  other
end

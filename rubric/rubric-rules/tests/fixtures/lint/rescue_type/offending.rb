begin
  foo
rescue nil
  bar
end

begin
  foo
rescue true
  bar
end

begin
  baz
rescue false
  qux
end

begin
  something
rescue 42
  other
end

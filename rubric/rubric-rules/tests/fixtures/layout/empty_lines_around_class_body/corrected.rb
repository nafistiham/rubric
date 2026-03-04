class Foo
  def bar
    1
  end
end

# Single-line nested class followed by empty line — must NOT be flagged
class Outer
  class Error < StandardError; end

  def foo
    1
  end
end

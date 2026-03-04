def foo
  1 + 1
end

# Single-line method definition followed by empty line — must NOT be flagged
class Foo
  def show; end

  def update
    do_something
  end
end

# Ruby 3 endless method definition followed by empty line — must NOT be flagged
class Bar
  def self.name = 'Bar'

  def to_s = name
end

Person = Struct.new(:name, :age)
person = Person.new('Alice', 30)

class Point
  attr_reader :x, :y
  def initialize(x, y)
    @x = x
    @y = y
  end
end

# OpenStruct.new is mentioned only in comments

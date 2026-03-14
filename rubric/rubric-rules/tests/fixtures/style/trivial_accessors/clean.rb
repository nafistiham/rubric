attr_reader :name
attr_writer :age
attr_accessor :title

def name
  @name.upcase
end

def complex_getter
  @data ||= compute
end

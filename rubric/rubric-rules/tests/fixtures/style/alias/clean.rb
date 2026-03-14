class Foo
  alias_method :new_name, :old_name
  alias_method :to_s, :inspect
end

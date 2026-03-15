# These should NOT be flagged

# Single-line ternary is fine
result = condition ? value_a : value_b

# if/unless is fine
result = if some_condition
  value_if_true
else
  value_if_false
end

# Predicate method call (ends with ?) is fine
if foo?
  bar
end

# Method call with ? at end of name
valid = obj.valid?
active = user.active?

hash = pairs.to_h
result = keys.zip(values).to_h
# Hash[:key] is not a constructor
value = my_hash[:key]

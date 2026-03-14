# These should not be flagged
users.intersect?(admins)
a.intersect?(b)
a.any? { |x| b.include?(x) }
result = arr.any?
# (a & b).any? is bad — comment only, not code

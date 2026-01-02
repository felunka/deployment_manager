Datagrid::Filters::StringFilter.module_eval do
  def default_filter_where(scope, value)
    scope.where("%s ILIKE '%s'", name, "%#{value}%")
  end
end

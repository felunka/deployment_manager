class NodesGrid < ApplicationGrid
  #
  # Scope
  #
  scope do
    Node
  end

  #
  # Filters
  #
  filter(:hostname, :string)
  filter(:ip, :string)
  filter(:last_name, :string)
  filter(:node_status, :enum, select: Node.distinct.pluck(:node_status).map { |type| [ I18n.t("simple_form.options.defaults.node_status.#{type}"), type ] }, multiple: true)

  #
  # Columns
  #
  column(:hostname)
  column(:ip)
  column(:node_status) { |asset| I18n.t("simple_form.options.defaults.node_status.#{asset.node_status}") }
  actions
end

class NodeDeploymentsGrid < ApplicationGrid
  #
  # Scope
  #
  scope do
    NodeDeployment
  end

  #
  # Filters
  #
  filter(:name, :string)
  filter(:path, :string)

  #
  # Columns
  #
  column(:node)
  column(:name)
  column(:path)
  column(:deployment_type) { |asset| I18n.t("simple_form.options.defaults.deployment_type.#{asset.deployment_type}") }
  column(:deployment_status) { |asset| I18n.t("simple_form.options.defaults.deployment_status.#{asset.deployment_status}") }
  actions(button_options: { form: { "data-turbo-frame": "_top" } })
end

class ContainerGrid < ApplicationGrid
  #
  # Scope
  #
  scope { [] }

  #
  # Filters
  #

  #
  # Columns
  #
  column(:id) { |asset| asset["Id"] }
  column(:image) { |asset| asset["Image"] }
  column(:command) { |asset| asset["Command"] }
  column(:created) { |asset| I18n.l Time.at(asset["Created"]) }
  column(:ports) { |asset| asset["Ports"].map { |port| "#{port['IP']}:#{port['PublicPort']}->#{port['PrivatePort']}/#{port['Type']}" }.join(", ") }
  column(:state) { |asset| "#{asset['State']} (#{asset['Status']})" }
  column(:name) { |asset| asset["Names"].join(", ") }
  column(:actions, html: true, header: "") do |asset|
    content_tag(:div, class: "d-flex model-buttons") do
      concat(
        button_to node_container_path(node_id: asset[:node_id], id: asset["Id"]), class: "btn btn-primary btn-sm", method: :get, form: { "data-turbo-frame": "_top" } do
          icon "search"
        end
      )
      concat(
        button_to logs_node_container_path(node_id: asset[:node_id], id: asset["Id"]), class: "btn btn-primary btn-sm", method: :get, form: { "data-turbo-frame": "_top" } do
          icon "card-list"
        end
      )
    end
  end
end

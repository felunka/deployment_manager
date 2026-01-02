class ApplicationGrid < Datagrid::Base
  def header_label(column)
    if column.options.key? :header
      column.options[:header]
    else
      model_name_key = translation_model_name || model_name.i18n_key.to_s.gsub("_grid", "").singularize
      I18n.t("activerecord.attributes.#{model_name_key}.#{column.name}")
    end
  end

  def translation_model_name
    nil
  end

  def self.actions(available_actions: [ :show, :edit, :destroy ], button_options: {})
    column(:actions, html: true, header: "") do |asset|
      content_tag(:div, class: "d-flex model-buttons") do
        if available_actions.include? :show
          concat(
            button_to(send("#{asset.model_name.singular}_path", asset), class: "btn btn-primary btn-sm", method: :get, **button_options) do
              icon "search"
            end
          )
        end
        if available_actions.include? :edit
          concat(
            button_to(send("edit_#{asset.model_name.singular}_path", asset), class: "btn btn-primary btn-sm", method: :get, **button_options) do
              icon "pen"
            end
          )
        end
        if available_actions.include? :destroy
          concat(
            button_to(send("#{asset.model_name.singular}_path", asset), class: "btn btn-danger btn-sm", method: :delete, form: { data: { turbo_confirm: t("helpers.submit.confirm") } }, **button_options) do
              icon "trash"
            end
          )
        end
      end
    end
  end
end

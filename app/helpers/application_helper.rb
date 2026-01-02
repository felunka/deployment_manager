module ApplicationHelper
  def nav_item(name, path, admin = false)
    return if current_user.nil?
    content_tag :li, class: "nav-item #{'active' if current_page?(path)}" do
      link_to name, path, class: "nav-link"
    end
  end

  def icon(name, classes = [])
    tag_classes = [ "bi", "bi-#{name}" ] + classes
    content_tag :i, nil, class: tag_classes
  end

  def current_translations
    @translations ||= I18n.backend.send(:translations)
    @translations[I18n.locale].with_indifferent_access
  end
end

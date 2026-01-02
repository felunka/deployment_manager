class ApplicationController < ActionController::Base
  # Only allow modern browsers supporting webp images, web push, badges, import maps, CSS nesting, and CSS :has.
  allow_browser versions: :modern

  # Changes to the importmap will invalidate the etag for HTML responses
  stale_when_importmap_changes

  before_action :require_login

  helper_method :current_user

  NotAuthorized = Class.new(StandardError)

  def current_user
    return nil unless session[:user_id]
    @current_user ||= User.find session[:user_id]
  end

  def require_login
    unless current_user
      redirect_to root_path
    end
  end

  def require_admin
    if current_user.nil? || !current_user.admin?
      redirect_to root_path
    end
  end

  rescue_from ApplicationController::NotAuthorized do |exception|
    respond_to do |format|
      format.any  { head 403 }
    end
  end
end

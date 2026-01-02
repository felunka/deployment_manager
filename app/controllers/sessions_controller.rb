class SessionsController < ApplicationController
  skip_before_action :require_login, only: [ :new, :create ]

  def new
    @token = params[:token]
  end

  def create
    user = User.find_by(params.require(:user).permit(:email))

    respond_to do |format|
      if user && user.authenticate(params.require(:user)[:password])
        reset_session
        session[:user_id] = user.id
        flash[:success] = t("messages.login.success")
        format.html { redirect_to nodes_path }
      else
        flash[:danger] = t("messages.login.fail")
        format.html { redirect_to action: "new", status: :unprocessable_entity }
      end
    end
  end

  def destroy
    reset_session

    redirect_to action: "new"
  end
end

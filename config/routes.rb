Rails.application.routes.draw do
  # Define your application routes per the DSL in https://guides.rubyonrails.org/routing.html

  root "nodes#index"

  resource :session, only: [ :new, :create, :destroy ]
  resources :nodes do
    member do
      get :health
    end

    resources :container, only: [ :index, :show ] do
      member do
        get :logs
        get "/action/:action_name", to: "container#action", as: :action
      end
    end
    resources :node_deployments
  end
  resources :node_deployments do
    member do
      get :status
    end
  end
end

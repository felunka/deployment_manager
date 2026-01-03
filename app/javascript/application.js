// Configure your import map in config/importmap.rb. Read more: https://github.com/rails/importmap-rails
import "@hotwired/turbo-rails"
import "controllers"
import 'select2'
import "@popperjs/core"
import * as bootstrap from "bootstrap"

import BootstrapTooltips from 'components/bootstrap_tooltips'
import Select2 from 'components/select2'
import GenericAjaxErrorDisplay from 'components/generic_anjax_error_display'

// Fix popperjs
window.process = {};
window.process.env = {};
window.process.env.NODE_ENV = "production";

// Load translations
window.I18n = JSON.parse(document.getElementById("i18n").textContent);

document.addEventListener("turbo:load", function() {
  BootstrapTooltips.setup();
  Select2.setup();
  GenericAjaxErrorDisplay.setup();
});


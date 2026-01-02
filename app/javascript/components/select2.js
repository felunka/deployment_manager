import jQuery from "jquery"
import 'select2'

export default class Select2 {
  static setup() {
    $('.js-states').select2();
    $('.js-states-autocomplete').each(function() {
      Select2.initSelectAutocomplete(this);
    });
  }

  static initSelectAutocomplete(element) {
    let url = $(element).data('url');
    $(element).select2({
      minimumInputLength: 2,
      allowClear: true,
      ajax: {
        url: url,
        processResults: function(data) {
          return {
            results: data
          };
        }
      }
    });
  }
};

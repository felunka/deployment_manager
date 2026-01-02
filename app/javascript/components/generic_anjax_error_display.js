export default class GenericAjaxErrorDisplay {
  static setup() {
    $(document).on('ajax:error', function(xhr, status, error) {
      let alert = document.createElement('div');
      alert.className = 'alert alert-dissmissible mt-3 alert-danger';
      alert.innerText = xhr.detail[0];
      $('#flashbox').append(alert);
    });
  }
};

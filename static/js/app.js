'use strict';

angular.module('underscore', [])
.factory('_', function() {
    return window._; // assumes underscore has already been loaded on the page
});

angular.module('expensesApp', 
    ['ui.bootstrap', 'underscore', 'expensesApp.directives', 
    'expensesApp.controllers', 'expensesServices'])
.config(function($httpProvider, $routeProvider, $locationProvider) {
  //$locationProvider.html5Mode(false);

  var greyedOut = function(innerHtml) {
    return $('<div style="position:fixed;top:0;left:0;right:0;bottom:0;z-index:10000;background-color:gray;background-color:rgba(70,70,70,0.2);">' + 
      innerHtml + '</div>').appendTo($('body')).hide();
  };

  var loadingScreen = greyedOut('<div class="center"><div>Loading</div><div><img src="images/ajax-loader.gif"/></div></div>');

  $routeProvider
    .when('/', { templateUrl: './partials/expense-table.html' })
    .when('/index.html', { templateUrl: './partials/expense-table.html' })
    .when('/expenses', { templateUrl: './partials/expense-table.html' })
    .when('/graph/expenses/:name', { templateUrl: './partials/bar-graph.html', controller: 'ExpenseGraphController' })
    .when('/graph/:year/:month', { templateUrl: './partials/pie-graph.html', controller: 'MonthGraphController' })
    .when('/graph/total', { templateUrl: './partials/pie-graph.html', controller: 'TotalGraphController' })

  var numLoadings = 0;

  $httpProvider.responseInterceptors.push(function($timeout, $q) {
    return function(promise) {
      // Taken from http://blog.tomaka17.com/2012/12/random-tricks-when-using-angularjs/
      numLoadings++;
      loadingScreen.show();

      var hide = function(r) { 
        if (!(--numLoadings)) 
          loadingScreen.hide(); 
          return r; 
      };

      var error = function(errorResponse) {
        loadingScreen.hide();
        var msg = "Please contact support.";
        if (errorResponse.data !== "") {
          msg = errorResponse.data;
        } else if (errorResponse.status === 404) {
          msg = errorResponse.config.url + " does not exist.";
        }
        $('<div class="alert alert-error">An HTTP error occurred.  ' + msg + '</div>').prependTo($('body'));
        return $q.reject(errorResponse);
      }
      return promise.then(hide, error);
    };
  });
});

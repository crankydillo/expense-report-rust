angular.module('expensesServices', ['underscore'])
.factory('MonthlyExpenses', function($http, $location) {
  var addParams = function(params) {
    var url = arguments[0];
    var params = "";
    for (var i = 1; i < arguments.length; i++) {
      var name = arguments[i];
      var value = $location.search()[name];
      if (typeof value != 'undefined') {
        params = params + "&" + name + "=" + value;
      }
    }
    if (params.length == 0) {
      return url;
    }
    if (url.indexOf("?") == -1) {
      return url + "?" + params.slice(1);
    } else {
      return url + params;
    }
  };

  var get = function(url) {
    // a promise
    return $http.get(url).then(function(data) {
      return data.data;
    });
  };

  /**
   * Retrieve a monthly breakdown.
   *
   * @param processingFn a function to apply to the retrieved data.
   */
  var monthlyBreakdown = function(scope, processingFn) {
    // TODO Read about null/undefined checks
    var url = addParams('/res/monthly-totals', 'months', 'since', 'until', 'q');
    return $http.get(url).then(function(data) {
        console.log("args: " + arguments.length);
        if (processingFn) {
            return processingFn(data.data);
        }
        return data.data;
    });
  };

  var splits = function(name, month) {
    var url = addParams("/res/expenses/" + name + "/" + month, 'q');
    return get(url);
  };

  var expenseBreakdown = function(year, month) {
    var qualifiedName = function(acct) {
      if (typeof(acct.parent) != "undefined" && !(acct.parent.name === "Expenses")) {
        return qualifiedName(acct.parent) + ":" + acct.name;
      }
      return acct.name;
    }

    var url = "/res/expenses/" + year + "/" + month;

    return $http.get(url).then(function(data) {
      return _.map(data.data, function(at) { 
        var nm = qualifiedName(at.account);
        return [nm, at.total];
      });
    });
  };

  return {
    monthlyBreakdown: monthlyBreakdown,
    expenseBreakdown: expenseBreakdown,
    splits: splits
  };
})

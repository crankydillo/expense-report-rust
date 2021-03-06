'use strict';

angular.module('expensesApp.controllers', ['underscore'])
.controller('ExpensesController', function($scope, $http, $location, $filter, MonthlyExpenses) {

  $scope.$on('loadsplits', function (event, expense) {
    $scope.expense = expense;
  });

  $scope.fmtPercent = fmtPercent;
  $scope.fmtMoney = fmtMoney;

  $scope.openHelp = function () {
    $scope.showHelp = true;
  };

  $scope.closeHelp = function () {
    $scope.showHelp = false;
  };

  $scope.helpOps = {
    backdropFade: true,
    dialogFade:true
  };

  // I have to figure out how to create resuable functions.  Until then, I'm
  // probably abusing this 'parent' controller
  $scope.resetSplits = function(msg) {
    $('#exp-splits').hide();
    $('#splits-intro').show();  // TODO Not supposed to do this jQuery stuff in controllers
    $scope.splits = {
      msg: msg
    };
  }

  $scope.mkSlices = function(data, maxSlices) {
    if (data.length < maxSlices) {
      return data;
    }
    var numReg = maxSlices - 1;
    var others = _.rest(data, numReg);
    var otherValues = _.map(others, function(o) { return o[1]; });
    var sumOthers = _.reduce(otherValues, function(sum, c) { return sum + c }, 0);
    var arr = _.first(data, numReg);
    arr.push(['Other', Math.round(sumOthers * 100) / 100]);
    return arr;
  }

  $scope.$on('$routeChangeSuccess', function() {
    $scope.resetSplits('Click on a cell to see its transactions.');
  });

  $scope.selected = function(rowIdx, colIdx) {
    $scope.selectedRow = rowIdx;
    $scope.selectedCol = colIdx;
  }

  $scope.loadSplits = function(name, monthSummary) {
    $('#splits-intro').hide();  // TODO Not supposed to do this jQuery stuff in controllers
    $('#exp-splits').show();

    MonthlyExpenses.splits(name, monthSummary.month, $scope).then(function(splits) {
      $scope.expense = splitsToExp(name, splits);
    });
  };

  var maxAccts = $location.search().max;

  var processingFn = function(data) {
      var acctSums = data.acctSums;
      if (maxAccts) {
          var showAccts = acctSums.slice(0, maxAccts);
          var restAccts = acctSums.slice(maxAccts);
          var init = restAccts.shift();
          init.name = "Root Account:Expenses:Rest";
          var rest = _.reduce(restAccts, function(memo, acct) {
              for (var i = 0; i < memo.monthlyTotals.length; i++) {
                  var mt = memo.monthlyTotals[i];
                  var curr = acct.monthlyTotals[i];
                  mt.total = mt.total + curr.total;
              }
              memo.total = memo.total + acct.total;
              return memo;
          }, init);
          data.acctSums = showAccts.concat([rest]);
      } 
      return data;
  };

  //MonthlyExpenses.monthlyBreakdown(processingFn, $scope).then(function(mb) {
  MonthlyExpenses.monthlyBreakdown($scope, processingFn).then(function(mb) {
    mb.summaries.reverse();
    _.each(mb.acctSums, function(as) { return as.monthlyTotals.reverse(); });
    $scope.monthlyBreakdown = mb;
  });

  $scope.pad = pad;

  $scope.trimmedName = function(qualifiedName) {
    return qualifiedName.substring(22, qualifiedName.length);
  }
})

.controller('MonthGraphController', 
    function($scope, $routeParams, $http, MonthlyExpenses) {

  $scope.$on('$routeChangeSuccess', function() {
    $scope.$parent.resetSplits('Click on a slice to see its transactions.');
  });

  $scope.year = parseInt($routeParams.year);
  $scope.month = parseInt($routeParams.month);
  $scope.title = $scope.year + "-" + $scope.$parent.pad($scope.month, 2);

  MonthlyExpenses.expenseBreakdown($scope.year, $scope.month, $scope).then(function(data) {
    $scope.chartData = $scope.$parent.mkSlices(data, 10);
  });
})

.controller('TotalGraphController', function($scope, $routeParams, $http) {
  $scope.$on('$routeChangeSuccess', function() {
    $scope.$parent.resetSplits('Transaction list is not supported (yet).');
  });

  $scope.title = "Total";

  var acctSums = $scope.monthlyBreakdown.acctSums;

  var data = _.map(acctSums, function(as) { 
    var nm = $scope.$parent.trimmedName(as.name);
    return [nm, as.total];
  });

  $scope.chartData = $scope.$parent.mkSlices(data, 10);
})

.controller('ExpenseGraphController', function($scope, $routeParams, $http) {
  console.log('ExpenseGraphController');
  $scope.$on('$routeChangeSuccess', function() {
    $scope.$parent.resetSplits('Click on a bar to see its transactions.');
  });

  var expense = $routeParams.name;

  var row = _.find($scope.monthlyBreakdown.acctSums, function(as) { 
    return as.name === expense;
  });

  $scope.expenseName = $scope.$parent.trimmedName(expense);
  $scope.expenseTotal = row.total;
  $scope.chartData = row.monthlyTotals;
})

.controller('CatchAllController', function($location) {
  console.log($location.path());
  alert($location.path());
})

.controller('BudgetController', function($scope, $http, $window, $location, $filter, MonthlyExpenses) {

  $window.document.title = 'Budget';
  $scope.fmtMoney = fmtMoney;

  MonthlyExpenses.budget().then(function(budget) {
    var amountTypes;

    _.each(budget.amounts, function(a) {
      a.qualified_name = a.name;
      a.name = trimmedName(a.name);
    });
    budget.amounts = _.sortBy(budget.amounts, function(a) {
      return a.name;
    });

    amountTypes = _.partition(budget.amounts, function(a) {
      return a.in_budget;
    });
    

    budget.totals = {};
    budget.totals.actuals = {};

    budget.totals.budgeted = _.reduce(budget.amounts, function(memo, a) {
      return memo + a.budgeted;
    }, 0);

    budget.totals.actuals.total = _.reduce(budget.amounts, function(memo, a) {
      return memo + a.actual;
    }, 0);

    budget.amounts = {};
    budget.amounts.budgeted   = amountTypes[0];
    budget.amounts.unbudgeted = amountTypes[1];

    budget.totals.actuals.budgeted = _.reduce(budget.amounts.budgeted, function(memo, a) {
      return memo + a.actual;
    }, 0);

    budget.totals.actuals.unbudgeted = _.reduce(budget.amounts.unbudgeted, function(memo, a) {
      return memo + a.actual;
    }, 0);

    $scope.budget = budget;
  });

  $scope.loadBudgetSplits = function(acctName) {
    var now = new Date(),
        yearMo = (1900 + now.getYear()) + "-" + pad((now.getMonth() + 1) + "", 2);
    $('#splits-intro').hide();  // TODO Not supposed to do this jQuery stuff in controllers
    $('#exp-splits').show();

    MonthlyExpenses.splits(acctName, yearMo, $scope).then(function(splits) {
      $scope.$emit("loadsplits", splitsToExp(acctName, splits));
    });
  }
})
.controller('SearchController', function($scope, $http, $window, $location, $filter, MonthlyExpenses) {

  console.log('search called');
  $window.document.title = 'Search';
  $scope.fmtMoney = fmtMoney;

  $scope.search = function(query) {
    MonthlyExpenses.search(query).then(function(results) {
      $scope.search_results = results;
    });
  };
});


/*
 * Convert splits from the server into what UI wants.
 */
function splitsToExp(qualifiedName, splits) {
  var exp = {};
  exp.name = trimmedName(qualifiedName);
  exp.splits = splits.splits;
  return exp;
}

function fmtPercent(num, den) {
  if (den == 0 || num == 0) return "";
  return Math.round((num / den) * 100) + "%";
}

function fmtMoney(num) {
  return (num / 100.0) + "";
}

function trimmedName(qualifiedName) {
  return qualifiedName.substring(22, qualifiedName.length);
}

function pad(str, len) {
    str = str + "";
    if (str.length >= len)
      return str;
    return new Array(len - str.length + 1).join("0") + str;
  }


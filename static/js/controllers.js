'use strict';

angular.module('expensesApp.controllers', ['underscore'])
.controller('ExpensesController', function($scope, $http, $location, $filter, MonthlyExpenses) {

  $scope.fmtMoney = function(num) {
    return (num / 100.0) + "";
  }

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
      $scope.splits = splits.splits;
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

  $scope.pad = function(str, len) {
    str = str + "";
    if (str.length >= len)
      return str;
    return new Array(len - str.length + 1).join("0") + str;
  }

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
});

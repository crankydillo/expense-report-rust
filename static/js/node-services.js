'use strict';

var java = require('java');
var fs = require('fs');
var gui = require('nw.gui');

var jars = fs.readdirSync('./dist/app/lib/');

jars.forEach( function (file) {
  java.classpath.push("dist/app/lib/" + file);
});

/*
 * @description
 * A factory that creates a gnucash service.  This service allows client code to
 * interact with gnucash by utilizing node-webkit to call into a Scala gnucash
 * API.
 */
angular.module('expensesServices', [])
.factory('MonthlyExpenses', function($q) {

  var scalaMod = function(nm) {
    return java.getStaticFieldValue(nm, 'MODULE$');
  }

  console.log(gui.App.argv);
  if (gui.App.argv.length != 1) {
    console.log("Please enter the path to the gnucash data file.");
    gui.App.quit();
  }
  var dataFilePath = gui.App.argv[0];

  var dataFile = java.newInstanceSync('java.io.File', dataFilePath);

  if (!dataFile.isFileSync()) {
    console.log(dataFile.getAbsolutePathSync() + " must exist and be a file.")
    gui.App.quit();
  }

  var accountDao = scalaMod('org.beeherd.gnucash.dao.AccountDAO$').applySync(dataFile);
  var transDao = scalaMod('org.beeherd.gnucash.dao.TransactionDAO$').applySync(
      dataFile, accountDao);

  var expSvc = java.newInstanceSync('controllers.ExpensesService',
    accountDao, transDao);

  var promise = function(wrappedFn, fnArgs, scope, successFn) {
    var deferred = $q.defer();

    var apiCallback = function(err, result) {
      if (err) { 
        var msg = "API error: " + err;
        deferred.reject(msg);
        return;
      }
      try {
        if (!successFn) {
          successFn = function(r) { 
            return JSON.parse(r);
          }
        }
        deferred.resolve(successFn(result));
      } catch (err) {
        var msg2 = "Cannot parse: " + result;
        console.log(msg2);
        deferred.reject(msg2);
      }
      if (!scope.$$phase) {
        scope.$apply();
      }
    };

    fnArgs.push(apiCallback);

    wrappedFn.apply(expSvc, fnArgs);
    return deferred.promise;
  };

  var monthlyBreakdown = function(scope) {
    var apiFn = expSvc.monthlyTotalsAsJSON;
    return promise(apiFn, [6], scope);
  };

  var splits = function(qualifiedName, year, month, scope) {
    var apiFn = expSvc.accountListAsJSON;
    var fnArgs = [qualifiedName, year, month];
    return promise(apiFn, fnArgs, scope);
  };

  var expenseBreakdown = function(year, month, scope) {
    // TODO Code duplicated in (restws-)services.js
    var qualifiedName = function(acct) {
      if (typeof(acct.parent) != "undefined" && !(acct.parent.name === "Expenses")) {
        return qualifiedName(acct.parent) + ":" + acct.name;
      }
      return acct.name;
    }

    var apiFn = expSvc.monthListAsJSON;
    var fnArgs = [year, month];

    var successFn = function(result) {
      var acctTotals = JSON.parse(result);
      return _.map(acctTotals, function(at) { 
        var nm = qualifiedName(at.account);
        return [nm, at.total];
      });
    };

    return promise(apiFn, fnArgs, scope, successFn);
  };

  return {
    monthlyBreakdown: monthlyBreakdown
    , splits: splits
    , expenseBreakdown: expenseBreakdown
  };
});

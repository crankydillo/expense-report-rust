'use strict';

angular.module('expensesApp.directives', [])
.directive('graph', function() {
  
  return {
    restrict: 'A',
    link: function(scope, elm, attrs) {
      scope.$watch("chartData", function(data) {
        $('#container').highcharts({
          chart: {
            plotBackgroundColor: null,
            plotBorderWidth: null,
            plotShadow: false
          },
          title: {
            text: scope.title
          },
          tooltip: {
            pointFormat: "${point.y} ({point.percentage:.1f}%)"
          },
          plotOptions: {
            pie: {
              allowPointSelect: true,
              cursor: 'pointer',
              dataLabels: {
                enabled: true,
                color: '#000000',
                connectorColor: '#000000',
                format: '<b>{point.name}</b>: {point.percentage:.1f}%'
              }
            }
          },
          series: [{
            type: 'pie',
            name: 'Expenses',
            point: {
              events: {
                click: function() {
                  if (this.name === "Other") {
                    scope.$parent.resetSplits("Transaction list for 'Other' is not supported (yet).");
                  } else {
                    scope.$parent.loadSplits("Root Account:Expenses:" + this.name,
                        { year: scope.year, month: scope.month }
                      );
                  }
                }
              }
            },
            data: data
          }]
        });
      });
    }
  }
})

.directive('barGraph', function() {
  
  return {
    restrict: 'A',
    link: function(scope, elm, attrs) {
      scope.$watch("chartData", function(monthSummaries) {

        var categories = _.map(monthSummaries, function(ms) {
          return ms.year + "-" + ms.month;
        });

        var data = _.map(monthSummaries, function(ms) { return ms.total; });

        $('#container').highcharts({
          chart: {
            type: 'column'
          },
          title: {
            text: attrs.barGraph
          },
          xAxis: {
            categories: categories,
            title: {
                text: scope.expenseName
            }
          },
          yAxis:  {
            min: 0,
            title: {
              text: 'Dollars'
            }
          },
          tooltip: {
            pointFormat: '<span><b>${point.y}</b></span>'
          },
          series: [{
            name: scope.title,
            point: {
              events: {
                click: function() {
                  var arr = this.category.split("-");
                  scope.$parent.loadSplits("Root Account:Expenses:" + scope.expenseName,
                      { year: parseInt(arr[0]), month: parseInt(arr[1]) }, scope);
                }
              }
            },

            data: data
          }]
        });
      });
    }
  }
});

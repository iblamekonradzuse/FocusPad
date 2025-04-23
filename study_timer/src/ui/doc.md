Struct Chart
Settings
Help
Source

pub struct Chart { /* private fields */ }

The chart representation.
Anatomy of a Chart

A chart is a collection of different components, each of which is responsible for rendering a specific part of the chart. Below is a sample chart with a few components:

                   Sales Report
  |                                                        # coffee
30|                  x                                     x juice
  |      @           x             @                       @ milk
20|    # @           x@           x@          #
  |    #x@          #x@          #x@          #x
10|    #x@          #x@          #x@          #x@
  |    #x@          #x@          #x@          #x@
 0+-----------------------------------------------------
       Jan          Feb          Mar          Apr

The chart above has the following components: an x axis, an y axis, a title on the top center, and a legend on the top right.

The creation of charts in Charming is done in a builder-like fashion. Once you get a hang of this pattern, you will find that it is very easy to compose a chart. For instance, the following code snippet shows how to create the chart above:

use charming::Chart;
use charming::component::{Axis, Legend, Title};

let chart = Chart::new()
    .title(Title::new().text("Sales Report"))
    .x_axis(Axis::new().data(vec!["Jan", "Feb", "Mar", "Apr"]))
    .y_axis(Axis::new())
    .legend(Legend::new().data(vec!["coffee", "juice", "milk"]));

Components of a Chart

The following sections describe the components of a chart in detail.
Title

Title is the title of a chart, including main title and subtitle. A chart can have multiple titles, which is useful when you want to show multiple sub- charts in a single chart.

use charming::Chart;
use charming::component::Title;

let chart = Chart::new()
    .title(Title::new().text("Sales Report"));

Legend

Legend is the legend of a chart, which is used to show the meaning of the symbols and colors in the chart. A chart can have multiple legends.

use charming::Chart;
use charming::component::Legend;

let chart = Chart::new()
    .legend(Legend::new().data(vec!["coffee", "juice", "milk"]));

Grid

Grid is the background grid in a cartesian coordinate system. A chart can have multiple grids.

use charming::Chart;
use charming::component::Grid;

let chart = Chart::new()
    .grid(Grid::new());

X Axis and Y Axis

Axis is the axis in a cartesian coordinate system.

use charming::Chart;
use charming::component::Axis;

let chart = Chart::new()
    .x_axis(Axis::new().data(vec!["Jan", "Feb", "Mar", "Apr"]))
    .y_axis(Axis::new());

Polar Coordinate

[Polar] is the polar coordinate system. Polar coordinate can be used in scatter and line charts. Every polar coordinate has an AngleAxis and a RadiusAxis.
Radar Coordinate

RadarCoordinate is the radar coordinate system. Radar coordinate can be in radar charts.
Data Zoom

DataZoom is used for zooming a specific area, which enables user to view data in different scales. A chart can have multiple data zooms.
Visual Map

VisualMap is a visual encoding component. It maps data to visual channels, such as color, symbol size or symbol shape. A chart can have multiple visual maps.
Tooltip

Tooltip is a floating box that appears when user hovers over a data item.
AxisPointer

AxisPointer is a tool for displaying reference line and axis value under mouse pointer.
Toolbox

Toolbox is a feature toolbox that includes data view, save as image, data zoom, restore, and reset.
Implementations
Source
impl Chart
Source
pub fn new() -> Self
Source
pub fn title(self, title: Title) -> Self
Source
pub fn tooltip(self, tooltip: Tooltip) -> Self
Source
pub fn legend(self, legend: Legend) -> Self
Source
pub fn toolbox(self, toolbox: Toolbox) -> Self
Source
pub fn grid(self, grid: Grid) -> Self
Source
pub fn grid3d(self, grid: Grid3D) -> Self
Source
pub fn x_axis(self, x_axis: Axis) -> Self
Source
pub fn x_axis3d(self, x_axis: Axis3D) -> Self
Source
pub fn y_axis(self, y_axis: Axis) -> Self
Source
pub fn y_axis3d(self, y_axis: Axis3D) -> Self
Source
pub fn z_axis3d(self, z_axis: Axis3D) -> Self
Source
pub fn polar(self, polar: PolarCoordinate) -> Self
Source
pub fn angle_axis(self, angle_axis: AngleAxis) -> Self
Source
pub fn radius_axis(self, radius_axis: RadiusAxis) -> Self
Source
pub fn single_axis(self, single_axis: SingleAxis) -> Self
Source
pub fn parallel_axis(self, parallel_axis: ParallelAxis) -> Self
Source
pub fn axis_pointer(self, axis_pointer: AxisPointer) -> Self
Source
pub fn visual_map(self, visual_map: VisualMap) -> Self
Source
pub fn data_zoom(self, data_zoom: DataZoom) -> Self
Source
pub fn parallel(self, parallel: ParallelCoordinate) -> Self
Source
pub fn dataset(self, dataset: Dataset) -> Self
Source
pub fn radar(self, radar: RadarCoordinate) -> Self
Source
pub fn color(self, color: Vec<Color>) -> Self
Source
pub fn background_color<C: Into<Color>>(self, color: C) -> Self
Source
pub fn mark_line(self, mark_line: MarkLine) -> Self
Source
pub fn aria(self, aria: Aria) -> Self
Source
pub fn series<S: Into<Series>>(self, series: S) -> Self
Source
pub fn geo_map<M: Into<GeoMap>>(self, map: M) -> Self
Source
pub fn save_as_image_type(&self) -> Option<&SaveAsImageType>
Trait Implementations
Source
impl Default for Chart
Source
fn default() -> Self
Returns the “default value” for a type. Read more
Source
impl Display for Chart
Source
fn fmt(&self, f: &mut Formatter<'_>) -> Result
Formats the value using the given formatter. Read more
Source
impl Serialize for Chart
Source
fn serialize<__S>(&self, __serializer: __S) -> Result<__S::Ok, __S::Error>
where
    __S: Serializer,
Serialize this value into the given Serde serializer. Read more
Auto Trait Implementations
impl Freeze for Chart
impl RefUnwindSafe for Chart
impl Send for Chart
impl Sync for Chart
impl Unpin for Chart
impl UnwindSafe for Chart
Blanket Implementations
Source
impl<T> Any for T
where
    T: 'static + ?Sized,
Source
impl<T> Borrow<T> for T
where
    T: ?Sized,
Source
impl<T> BorrowMut<T> for T
where
    T: ?Sized,
Source
impl<T> Conv for T
Source
impl<T> FmtForward for T
Source
impl<T> From<T> for T

Source
impl<T, U> Into<U> for T
where
    U: From<T>,

Source
impl<T> IntoEither for T
Source
impl<T> Pipe for T
where
    T: ?Sized,
Source
impl<T> Pointable for T
Source
impl<R, P> ReadPrimitive<R> for P
where
    R: Read + ReadEndian<P>,
    P: Default,
Source
impl<T> Tap for T
Source
impl<T> ToString for T
where
    T: Display + ?Sized,
Source
impl<T> TryConv for T
Source
impl<T, U> TryFrom<U> for T
where
    U: Into<T>,
Source
impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
Source
impl<T> ErasedDestructor for T
where
    T: 'static,
Source
impl<T> MaybeSendSync for T

example code for line: 
use charming::{component::Axis, element::AxisType, series::Line, Chart};

pub fn chart() -> Chart {
    Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
        )
        .y_axis(Axis::new().type_(AxisType::Value))
        .series(Line::new().data(vec![150, 230, 224, 218, 135, 147, 260]))
}







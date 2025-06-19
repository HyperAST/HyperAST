use hyperast::test_utils::simple_tree::SimpleTree;

/// Example using the datasets/custom/{}/simple_class.java with the tree given by gumtree
pub(crate) fn example_from_gumtree_java_simple() -> (SimpleTree<u8>, SimpleTree<u8>) {
    // Parse the two Java files
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4), // type_body
            ]),
    ]);
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
    ]);
    (src_tr, dst_tr)
}

pub(crate) fn example_from_gumtree_java_method() -> (SimpleTree<u8>, SimpleTree<u8>) {
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test2"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "c"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(14; [ // local_variable_declaration
                                tree!(6, "int"), // type
                                tree!(15; [ // variable_declarator
                                    tree!(3, "b"), // identifier
                                    tree!(16, "="), // affectation_operator
                                    tree!(12; [ // binary_expression
                                        tree!(3, "c"), // identifier
                                        tree!(13, "*"), // arithmetic_operator
                                        tree!(17, "2"), // decimal_integer_literal
                                    ]),
                                ]),
                            ]),
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    (src_tr, dst_tr)
}

pub(crate) fn example_histogram_matching() -> (SimpleTree<u8>, SimpleTree<u8>) {
    // Parse the two Java files
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4), // type_body
            ]),
    ]);
    let dst_tr = tree!(
        0; [ // program
            tree!(6; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
    ]);
    (src_tr, dst_tr)
}

pub(crate) fn example_reorder_children() -> (SimpleTree<u8>, SimpleTree<u8>) {
    // Parse the two Java files
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test2"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "c"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
        ]
    );
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test2"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "c"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    (src_tr, dst_tr)
}

pub(crate) fn example_move_method() -> (SimpleTree<u8>, SimpleTree<u8>) {
    // Parse the two Java files
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                    tree!(5; [ // method_declaration
                        tree!(6, "String"), // type
                        tree!(3, "stuff"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                    ]),
                ]),
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
        ]
    );
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier - CHANGED
                tree!(4; [ // type_body
                     tree!(5; [ // method_declaration
                        tree!(6, "String"), // type
                        tree!(3, "stuff"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                    ]),
                ]), // type_body
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    (src_tr, dst_tr)
}

pub(crate) fn example_chart_1() -> (SimpleTree<u8>, SimpleTree<u8>) {
    let src_tr = tree!(1; [ // program
  tree!(2, "\\n * JFreeChart : a free chart library for the Java(tm) platform\\n * ===========================================================\\n *\\n * (C) Copyright 2000-2010, by Object Refinery Limited and Contributors.\\n *\\n * Project Info:  http://www.jfree.org/jfreechart/index.html\\n *\\n * This library is free software; you can redistribute it and/or modify it\\n * under the terms of the GNU Lesser General Public License as published by\\n * the Free Software Foundation; either version 2.1 of the License, or\\n * (at your option) any later version.\\n *\\n * This library is distributed in the hope that it will be useful, but\\n * WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY\\n * or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public\\n * License for more details.\\n *\\n * You should have received a copy of the GNU Lesser General Public\\n * License along with this library; if not, write to the Free Software\\n * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301,\\n * USA.\\n *\\n * [Java is a trademark or registered trademark of Sun Microsystems, Inc.\\n * in the United States and other countries.]\\n *\\n * ---------------------------------\\n * AbstractCategoryItemRenderer.java\\n * ---------------------------------\\n * (C) Copyright 2002-2010, by Object Refinery Limited.\\n *\\n * Original Author:  David Gilbert (for Object Refinery Limited);\\n * Contributor(s):   Richard Atkinson;\\n *                   Peter Kolb (patch 2497611);\\n *\\n * Changes:\\n * --------\\n * 29-May-2002 : Version 1 (DG);\\n * 06-Jun-2002 : Added accessor methods for the tool tip generator (DG);\\n * 11-Jun-2002 : Made constructors protected (DG);\\n * 26-Jun-2002 : Added axis to initialise method (DG);\\n * 05-Aug-2002 : Added urlGenerator member variable plus accessors (RA);\\n * 22-Aug-2002 : Added categoriesPaint attribute, based on code submitted by\\n *               Janet Banks.  This can be used when there is only one series,\\n *               and you want each category item to have a different color (DG);\\n * 01-Oct-2002 : Fixed errors reported by Checkstyle (DG);\\n * 29-Oct-2002 : Fixed bug where background image for plot was not being\\n *               drawn (DG);\\n * 05-Nov-2002 : Replaced references to CategoryDataset with TableDataset (DG);\\n * 26-Nov 2002 : Replaced the isStacked() method with getRangeType() (DG);\\n * 09-Jan-2003 : Renamed grid-line methods (DG);\\n * 17-Jan-2003 : Moved plot classes into separate package (DG);\\n * 25-Mar-2003 : Implemented Serializable (DG);\\n * 12-May-2003 : Modified to take into account the plot orientation (DG);\\n * 12-Aug-2003 : Very minor javadoc corrections (DB)\\n * 13-Aug-2003 : Implemented Cloneable (DG);\\n * 16-Sep-2003 : Changed ChartRenderingInfo --> PlotRenderingInfo (DG);\\n * 05-Nov-2003 : Fixed marker rendering bug (833623) (DG);\\n * 21-Jan-2004 : Update for renamed method in ValueAxis (DG);\\n * 11-Feb-2004 : Modified labelling for markers (DG);\\n * 12-Feb-2004 : Updated clone() method (DG);\\n * 15-Apr-2004 : Created a new CategoryToolTipGenerator interface (DG);\\n * 05-May-2004 : Fixed bug (948310) where interval markers extend outside axis\\n *               range (DG);\\n * 14-Jun-2004 : Fixed bug in drawRangeMarker() method - now uses 'paint' and\\n *               'stroke' rather than 'outlinePaint' and 'outlineStroke' (DG);\\n * 15-Jun-2004 : Interval markers can now use GradientPaint (DG);\\n * 30-Sep-2004 : Moved drawRotatedString() from RefineryUtilities\\n *               --> TextUtilities (DG);\\n * 01-Oct-2004 : Fixed bug 1029697, problem with label alignment in\\n *               drawRangeMarker() method (DG);\\n * 07-Jan-2005 : Renamed getRangeExtent() --> findRangeBounds() (DG);\\n * 21-Jan-2005 : Modified return type of calculateRangeMarkerTextAnchorPoint()\\n *               method (DG);\\n * 08-Mar-2005 : Fixed positioning of marker labels (DG);\\n * 20-Apr-2005 : Added legend label, tooltip and URL generators (DG);\\n * 01-Jun-2005 : Handle one dimension of the marker label adjustment\\n *               automatically (DG);\\n * 09-Jun-2005 : Added utility method for adding an item entity (DG);\\n * ------------- JFREECHART 1.0.x ---------------------------------------------\\n * 01-Mar-2006 : Updated getLegendItems() to check seriesVisibleInLegend\\n *               flags (DG);\\n * 20-Jul-2006 : Set dataset and series indices in LegendItem (DG);\\n * 23-Oct-2006 : Draw outlines for interval markers (DG);\\n * 24-Oct-2006 : Respect alpha setting in markers, as suggested by Sergei\\n *               Ivanov in patch 1567843 (DG);\\n * 30-Nov-2006 : Added a check for series visibility in the getLegendItem()\\n *               method (DG);\\n * 07-Dec-2006 : Fix for equals() method (DG);\\n * 22-Feb-2007 : Added createState() method (DG);\\n * 01-Mar-2007 : Fixed interval marker drawing (patch 1670686 thanks to\\n *               Sergei Ivanov) (DG);\\n * 20-Apr-2007 : Updated getLegendItem() for renderer change, and deprecated\\n *               itemLabelGenerator, toolTipGenerator and itemURLGenerator\\n *               override fields (DG);\\n * 18-May-2007 : Set dataset and seriesKey for LegendItem (DG);\\n * 20-Jun-2007 : Removed deprecated code and removed JCommon dependencies (DG);\\n * 27-Jun-2007 : Added some new methods with 'notify' argument, renamed\\n *               methods containing 'ItemURL' to just 'URL' (DG);\\n * 06-Jul-2007 : Added annotation support (DG);\\n * 17-Jun-2008 : Apply legend shape, font and paint attributes (DG);\\n * 26-Jun-2008 : Added crosshair support (DG);\\n * 25-Nov-2008 : Fixed bug in findRangeBounds() method (DG);\\n * 14-Jan-2009 : Update initialise() to store visible series indices (PK);\\n * 21-Jan-2009 : Added drawRangeLine() method (DG);\\n * 28-Jan-2009 : Updated for changes to CategoryItemRenderer interface (DG);\\n * 27-Mar-2009 : Added new findRangeBounds() method to account for hidden\\n *               series (DG);\\n * 01-Apr-2009 : Added new addEntity() method (DG);\\n * 09-Feb-2010 : Fixed bug 2947660 (DG);\\n *\\n */"), // block_comment
  tree!(3; [ // package_declaration
    tree!(4, "package"), // package
    tree!(5, "org.jfree.chart.renderer.category"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.AlphaComposite"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Composite"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Font"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.GradientPaint"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Graphics2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Paint"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Rectangle"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Shape"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Stroke"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Ellipse2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Line2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Point2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Rectangle2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.Serializable"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.ArrayList"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Iterator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.List"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.ChartRenderingInfo"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.LegendItem"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.LegendItemCollection"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.RenderingSource"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.annotations.CategoryAnnotation"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.axis.CategoryAxis"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.axis.ValueAxis"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.entity.CategoryItemEntity"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.entity.EntityCollection"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.event.RendererChangeEvent"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.CategoryItemLabelGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.CategorySeriesLabelGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.CategoryToolTipGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.ItemLabelPosition"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.StandardCategorySeriesLabelGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.CategoryCrosshairState"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.CategoryMarker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.CategoryPlot"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.DrawingSupplier"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.IntervalMarker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.Marker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.PlotOrientation"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.PlotRenderingInfo"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.ValueMarker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.renderer.AbstractRenderer"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.text.TextUtilities"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.urls.CategoryURLGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.GradientPaintTransformer"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.Layer"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.LengthAdjustmentType"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.ObjectList"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.ObjectUtilities"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.PublicCloneable"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.RectangleAnchor"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.RectangleEdge"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.RectangleInsets"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.SortOrder"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.Range"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.category.CategoryDataset"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.category.CategoryDatasetSelectionState"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.category.SelectableCategoryDataset"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.general.DatasetUtilities"), // identifier
  ]),
  tree!(2, "\\n * An abstract base class that you can use to implement a new\\n * {@link CategoryItemRenderer}.  When you create a new\\n * {@link CategoryItemRenderer} you are not required to extend this class,\\n * but it makes the job easier.\\n */"), // block_comment
  tree!(7; [ // type_declaration
    tree!(8; [ // modifiers
      tree!(9, "public"), // visibility
      tree!(10, "abstract"), // abstract
    ]),
    tree!(11, "class"), // type_keyword
    tree!(5, "AbstractCategoryItemRenderer"), // identifier
    tree!(12; [ // superclass
      tree!(13, "extends"), // extends
      tree!(14, "AbstractRenderer"), // type
    ]),
    tree!(15; [ // super_interfaces
      tree!(16, "implements"), // implements
      tree!(17; [ // type_list
        tree!(14, "CategoryItemRenderer"), // type
        tree!(14, "Cloneable"), // type
        tree!(14, "PublicCloneable"), // type
        tree!(14, "Serializable"), // type
      ]),
    ]),
    tree!(18; [ // type_body
      tree!(2, "/** For serialization. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
          tree!(20, "static"), // static
          tree!(21, "final"), // final
        ]),
        tree!(14, "long"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "serialVersionUID"), // identifier
          tree!(23, "="), // affectation_operator
          tree!(24, "1247553218442497391L"), // decimal_integer_literal
        ]),
      ]),
      tree!(2, "/** The plot that the renderer is assigned to. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryPlot"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "plot"), // identifier
        ]),
      ]),
      tree!(2, "/** A list of item label generators (one per series). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "ObjectList"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "itemLabelGeneratorList"), // identifier
        ]),
      ]),
      tree!(2, "/** The base item label generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "baseItemLabelGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** A list of tool tip generators (one per series). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "ObjectList"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "toolTipGeneratorList"), // identifier
        ]),
      ]),
      tree!(2, "/** The base tool tip generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "baseToolTipGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** A list of label generators (one per series). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "ObjectList"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "urlGeneratorList"), // identifier
        ]),
      ]),
      tree!(2, "/** The base label generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "baseURLGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** The legend item label generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "legendItemLabelGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** The legend item tool tip generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "legendItemToolTipGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** The legend item URL generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "legendItemURLGenerator"), // identifier
        ]),
      ]),
      tree!(2, "    \\n     * Annotations to be drawn in the background layer ('underneath' the data\\n     * items).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "List"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "backgroundAnnotations"), // identifier
        ]),
      ]),
      tree!(2, "    \\n     * Annotations to be drawn in the foreground layer ('on top' of the data\\n     * items).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "List"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "foregroundAnnotations"), // identifier
        ]),
      ]),
      tree!(2, "/** The number of rows in the dataset (temporary record). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
          tree!(25, "transient"), // transient
        ]),
        tree!(14, "int"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "rowCount"), // identifier
        ]),
      ]),
      tree!(2, "/** The number of columns in the dataset (temporary record). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
          tree!(25, "transient"), // transient
        ]),
        tree!(14, "int"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "columnCount"), // identifier
        ]),
      ]),
      tree!(2, "    \\n     * Creates a new renderer with no tool tip generator and no URL generator.\\n     * The defaults (no tool tip or URL generators) have been chosen to\\n     * minimise the processing required to generate a default chart.  If you\\n     * require tool tips or URLs, then you can easily add the required\\n     * generators.\\n     */"), // block_comment
      tree!(26; [ // constructor_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(5, "AbstractCategoryItemRenderer"), // identifier
        tree!(27), // formal_parameters
        tree!(28; [ // constructor_body
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "itemLabelGeneratorList"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ObjectList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "toolTipGeneratorList"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ObjectList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "urlGeneratorList"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ObjectList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemLabelGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "StandardCategorySeriesLabelGenerator"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "backgroundAnnotations"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ArrayList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "foregroundAnnotations"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ArrayList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the number of passes through the dataset required by the\\n     * renderer.  This method returns <code>1</code>, subclasses should\\n     * override if they need more passes.\\n     *\\n     * @return The pass count.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "getPassCount"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(24, "1"), // decimal_integer_literal
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the plot that the renderer has been assigned to (where\\n     * <code>null</code> indicates that the renderer is not currently assigned\\n     * to a plot).\\n     *\\n     * @return The plot (possibly <code>null</code>).\\n     *\\n     * @see #setPlot(CategoryPlot)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryPlot"), // type
        tree!(5, "getPlot"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "plot"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the plot that the renderer has been assigned to.  This method is\\n     * usually called by the {@link CategoryPlot}, in normal usage you\\n     * shouldn't need to call this method directly.\\n     *\\n     * @param plot  the plot (<code>null</code> not permitted).\\n     *\\n     * @see #getPlot()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setPlot"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "plot"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'plot' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "plot"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "plot"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// ITEM LABEL GENERATOR"), // line_comment
      tree!(2, "    \\n     * Returns the item label generator for a data item.  This implementation\\n     * returns the series item label generator if one is defined, otherwise\\n     * it returns the default item label generator (which may be\\n     * <code>null</code>).\\n     *\\n     * @param row  the row index (zero based).\\n     * @param column  the column index (zero based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(5, "getItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "CategoryItemLabelGenerator"), // type
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "itemLabelGeneratorList"), // identifier
                  ]),
                  tree!(5, "get"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "row"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "generator"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "baseItemLabelGenerator"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "generator"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the item label generator for a series.\\n     *\\n     * @param series  the series index (zero based).\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @see #setSeriesItemLabelGenerator(int, CategoryItemLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(5, "getSeriesItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(51; [ // cast_expression
              tree!(14, "CategoryItemLabelGenerator"), // type
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "itemLabelGeneratorList"), // identifier
                ]),
                tree!(5, "get"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the item label generator for a series and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getSeriesItemLabelGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setSeriesItemLabelGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the item label generator for a series and, if requested, sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getSeriesItemLabelGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "itemLabelGeneratorList"), // identifier
              ]),
              tree!(5, "set"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the base item label generator.\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @see #setBaseItemLabelGenerator(CategoryItemLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(5, "getBaseItemLabelGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "baseItemLabelGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item label generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getBaseItemLabelGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setBaseItemLabelGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item label generator and, if requested, sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getBaseItemLabelGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "baseItemLabelGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// TOOL TIP GENERATOR"), // line_comment
      tree!(2, "    \\n     * Returns the tool tip generator that should be used for the specified\\n     * item.  You can override this method if you want to return a different\\n     * generator per item.\\n     *\\n     * @param row  the row index (zero-based).\\n     * @param column  the column index (zero-based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(5, "getToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getSeriesToolTipGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "result"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "baseToolTipGenerator"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the tool tip generator for the specified series (a \"layer 1\"\\n     * generator).\\n     *\\n     * @param series  the series index (zero-based).\\n     *\\n     * @return The tool tip generator (possibly <code>null</code>).\\n     *\\n     * @see #setSeriesToolTipGenerator(int, CategoryToolTipGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(5, "getSeriesToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(51; [ // cast_expression
              tree!(14, "CategoryToolTipGenerator"), // type
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "toolTipGeneratorList"), // identifier
                ]),
                tree!(5, "get"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the tool tip generator for a series and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero-based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getSeriesToolTipGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setSeriesToolTipGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the tool tip generator for a series and sends a\\n     * {@link org.jfree.chart.event.RendererChangeEvent} to all registered\\n     * listeners.\\n     *\\n     * @param series  the series index (zero-based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getSeriesToolTipGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "toolTipGeneratorList"), // identifier
              ]),
              tree!(5, "set"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the base tool tip generator (the \"layer 2\" generator).\\n     *\\n     * @return The tool tip generator (possibly <code>null</code>).\\n     *\\n     * @see #setBaseToolTipGenerator(CategoryToolTipGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(5, "getBaseToolTipGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "baseToolTipGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base tool tip generator and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getBaseToolTipGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setBaseToolTipGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base tool tip generator and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getBaseToolTipGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "baseToolTipGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// URL GENERATOR"), // line_comment
      tree!(2, "    \\n     * Returns the URL generator for a data item.\\n     *\\n     * @param row  the row index (zero based).\\n     * @param column  the column index (zero based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return The URL generator.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(5, "getURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryURLGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "CategoryURLGenerator"), // type
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "urlGeneratorList"), // identifier
                  ]),
                  tree!(5, "get"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "row"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "generator"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "baseURLGenerator"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "generator"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the URL generator for a series.\\n     *\\n     * @param series  the series index (zero based).\\n     *\\n     * @return The URL generator for the series.\\n     *\\n     * @see #setSeriesURLGenerator(int, CategoryURLGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(5, "getSeriesURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(51; [ // cast_expression
              tree!(14, "CategoryURLGenerator"), // type
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "urlGeneratorList"), // identifier
                ]),
                tree!(5, "get"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the URL generator for a series and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator.\\n     *\\n     * @see #getSeriesURLGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setSeriesURLGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the URL generator for a series and, if requested, sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getSeriesURLGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "urlGeneratorList"), // identifier
              ]),
              tree!(5, "set"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the base item URL generator.\\n     *\\n     * @return The item URL generator.\\n     *\\n     * @see #setBaseURLGenerator(CategoryURLGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(5, "getBaseURLGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "baseURLGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item URL generator.\\n     *\\n     * @param generator  the item URL generator.\\n     *\\n     * @see #getBaseURLGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setBaseURLGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item URL generator.\\n     *\\n     * @param generator  the item URL generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @see #getBaseURLGenerator()\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "baseURLGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// ANNOTATIONS"), // line_comment
      tree!(2, "    \\n     * Adds an annotation and sends a {@link RendererChangeEvent} to all\\n     * registered listeners.  The annotation is added to the foreground\\n     * layer.\\n     *\\n     * @param annotation  the annotation (<code>null</code> not permitted).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addAnnotation"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAnnotation"), // type
            tree!(5, "annotation"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(49, "// defer argument checking"), // line_comment
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "addAnnotation"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "annotation"), // identifier
                tree!(31; [ // field_access
                  tree!(5, "Layer"), // identifier
                  tree!(5, "FOREGROUND"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Adds an annotation to the specified layer.\\n     *\\n     * @param annotation  the annotation (<code>null</code> not permitted).\\n     * @param layer  the layer (<code>null</code> not permitted).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addAnnotation"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAnnotation"), // type
            tree!(5, "annotation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Layer"), // type
            tree!(5, "layer"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "annotation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'annotation' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "layer"), // identifier
                tree!(5, "equals"), // identifier
                tree!(35; [ // argument_list
                  tree!(31; [ // field_access
                    tree!(5, "Layer"), // identifier
                    tree!(5, "FOREGROUND"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "foregroundAnnotations"), // identifier
                  ]),
                  tree!(5, "add"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "annotation"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(52; [ // method_invocation
                  tree!(5, "layer"), // identifier
                  tree!(5, "equals"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(5, "Layer"), // identifier
                      tree!(5, "BACKGROUND"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "backgroundAnnotations"), // identifier
                    ]),
                    tree!(5, "add"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "annotation"), // identifier
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "notifyListeners"), // identifier
                    tree!(35; [ // argument_list
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "RendererChangeEvent"), // type
                        tree!(35; [ // argument_list
                          tree!(32, "this"), // this
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(49, "// should never get here"), // line_comment
                tree!(45; [ // throw_statement
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "RuntimeException"), // type
                    tree!(35; [ // argument_list
                      tree!(46; [ // string_literal
                        tree!(47, "\""), // "
                        tree!(48, "Unknown layer."), // string_fragment
                        tree!(47, "\""), // "
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Removes the specified annotation and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @param annotation  the annotation to remove (<code>null</code> not\\n     *                    permitted).\\n     *\\n     * @return A boolean to indicate whether or not the annotation was\\n     *         successfully removed.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "boolean"), // type
        tree!(5, "removeAnnotation"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAnnotation"), // type
            tree!(5, "annotation"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "boolean"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "removed"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "foregroundAnnotations"), // identifier
                ]),
                tree!(5, "remove"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "annotation"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(5, "removed"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(42; [ // binary_expression
                tree!(5, "removed"), // identifier
                tree!(54, "&"), // bitwise_operator
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "backgroundAnnotations"), // identifier
                  ]),
                  tree!(5, "remove"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "annotation"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "notifyListeners"), // identifier
              tree!(35; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "RendererChangeEvent"), // type
                  tree!(35; [ // argument_list
                    tree!(32, "this"), // this
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "removed"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Removes all annotations and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "removeAnnotations"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "foregroundAnnotations"), // identifier
              ]),
              tree!(5, "clear"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "backgroundAnnotations"), // identifier
              ]),
              tree!(5, "clear"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "notifyListeners"), // identifier
              tree!(35; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "RendererChangeEvent"), // type
                  tree!(35; [ // argument_list
                    tree!(32, "this"), // this
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the legend item label generator.\\n     *\\n     * @return The label generator (never <code>null</code>).\\n     *\\n     * @see #setLegendItemLabelGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(5, "getLegendItemLabelGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "legendItemLabelGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the legend item label generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> not permitted).\\n     *\\n     * @see #getLegendItemLabelGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setLegendItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategorySeriesLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'generator' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemLabelGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "fireChangeEvent"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the legend item tool tip generator.\\n     *\\n     * @return The tool tip generator (possibly <code>null</code>).\\n     *\\n     * @see #setLegendItemToolTipGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(5, "getLegendItemToolTipGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "legendItemToolTipGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the legend item tool tip generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #setLegendItemToolTipGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setLegendItemToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategorySeriesLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemToolTipGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "fireChangeEvent"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the legend item URL generator.\\n     *\\n     * @return The URL generator (possibly <code>null</code>).\\n     *\\n     * @see #setLegendItemURLGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(5, "getLegendItemURLGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "legendItemURLGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the legend item URL generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getLegendItemURLGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setLegendItemURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategorySeriesLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemURLGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "fireChangeEvent"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the number of rows in the dataset.  This value is updated in the\\n     * {@link AbstractCategoryItemRenderer#initialise} method.\\n     *\\n     * @return The row count.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "getRowCount"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "rowCount"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the number of columns in the dataset.  This value is updated in\\n     * the {@link AbstractCategoryItemRenderer#initialise} method.\\n     *\\n     * @return The column count.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "getColumnCount"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "columnCount"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Creates a new state instance---this method is called from the\\n     * {@link #initialise(Graphics2D, Rectangle2D, CategoryPlot, int,\\n     * PlotRenderingInfo)} method.  Subclasses can override this method if\\n     * they need to use a subclass of {@link CategoryItemRendererState}.\\n     *\\n     * @param info  collects plot rendering info (<code>null</code> permitted).\\n     *\\n     * @return The new state instance (never <code>null</code>).\\n     *\\n     * @since 1.0.5\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "CategoryItemRendererState"), // type
        tree!(5, "createState"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "PlotRenderingInfo"), // type
            tree!(5, "info"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemRendererState"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "state"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "CategoryItemRendererState"), // type
                tree!(35; [ // argument_list
                  tree!(5, "info"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int[]"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "visibleSeriesTemp"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(55; [ // array_creation_expression
                tree!(34, "new"), // new
                tree!(14, "int"), // type
                tree!(56; [ // dimensions_expr
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "rowCount"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "visibleSeriesCount"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(24, "0"), // decimal_integer_literal
            ]),
          ]),
          tree!(57; [ // for_statement
            tree!(50; [ // local_variable_declaration
              tree!(14, "int"), // type
              tree!(22; [ // variable_declarator
                tree!(5, "row"), // identifier
                tree!(23, "="), // affectation_operator
                tree!(24, "0"), // decimal_integer_literal
              ]),
            ]),
            tree!(42; [ // binary_expression
              tree!(5, "row"), // identifier
              tree!(43, "<"), // comparison_operator
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "rowCount"), // identifier
              ]),
            ]),
            tree!(58; [ // update_expression
              tree!(5, "row"), // identifier
              tree!(59, "++"), // increment_operator
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(52; [ // method_invocation
                    tree!(5, "isSeriesVisible"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(60; [ // array_access
                        tree!(5, "visibleSeriesTemp"), // identifier
                        tree!(5, "visibleSeriesCount"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(5, "row"), // identifier
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(58; [ // update_expression
                      tree!(5, "visibleSeriesCount"), // identifier
                      tree!(59, "++"), // increment_operator
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int[]"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "visibleSeries"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(55; [ // array_creation_expression
                tree!(34, "new"), // new
                tree!(14, "int"), // type
                tree!(56; [ // dimensions_expr
                  tree!(5, "visibleSeriesCount"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "System"), // identifier
              tree!(5, "arraycopy"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "visibleSeriesTemp"), // identifier
                tree!(24, "0"), // decimal_integer_literal
                tree!(5, "visibleSeries"), // identifier
                tree!(24, "0"), // decimal_integer_literal
                tree!(5, "visibleSeriesCount"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "state"), // identifier
              tree!(5, "setVisibleSeriesArray"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "visibleSeries"), // identifier
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "state"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Initialises the renderer and returns a state object that will be used\\n     * for the remainder of the drawing process for a single chart.  The state\\n     * object allows for the fact that the renderer may be used simultaneously\\n     * by multiple threads (each thread will work with a separate state object).\\n     *\\n     * @param g2  the graphics device.\\n     * @param dataArea  the data area.\\n     * @param plot  the plot.\\n     * @param info  an object for returning information about the structure of\\n     *              the plot (<code>null</code> permitted).\\n     *\\n     * @return The renderer state.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemRendererState"), // type
        tree!(5, "initialise"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotRenderingInfo"), // type
            tree!(5, "info"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setPlot"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "plot"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "dataset"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "rowCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getRowCount"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "columnCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getColumnCount"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "rowCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(24, "0"), // decimal_integer_literal
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "columnCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(24, "0"), // decimal_integer_literal
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemRendererState"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "state"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "createState"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "info"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(49, "// determine if there is any selection state for the dataset"), // line_comment
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDatasetSelectionState"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "selectionState"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(5, "dataset"), // identifier
                tree!(62, "instanceof"), // instanceof
                tree!(14, "SelectableCategoryDataset"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "SelectableCategoryDataset"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "scd"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "SelectableCategoryDataset"), // type
                    tree!(5, "dataset"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "selectionState"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "scd"), // identifier
                    tree!(5, "getSelectionState"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(49, "// if the selection state is still null, go to the selection source"), // line_comment
          tree!(49, "// and ask if it has state..."), // line_comment
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(42; [ // binary_expression
                  tree!(5, "selectionState"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(44, "null"), // null_literal
                ]),
                tree!(63, "&&"), // logical_operator
                tree!(42; [ // binary_expression
                  tree!(5, "info"), // identifier
                  tree!(43, "!="), // comparison_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "ChartRenderingInfo"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "cri"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "info"), // identifier
                    tree!(5, "getOwner"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "cri"), // identifier
                    tree!(43, "!="), // comparison_operator
                    tree!(44, "null"), // null_literal
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "RenderingSource"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "rs"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "cri"), // identifier
                        tree!(5, "getRenderingSource"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "selectionState"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryDatasetSelectionState"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "rs"), // identifier
                          tree!(5, "getSelectionState"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "dataset"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "state"), // identifier
              tree!(5, "setSelectionState"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "selectionState"), // identifier
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "state"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the range of values the renderer requires to display all the\\n     * items from the specified dataset.\\n     *\\n     * @param dataset  the dataset (<code>null</code> permitted).\\n     *\\n     * @return The range (or <code>null</code> if the dataset is\\n     *         <code>null</code> or empty).\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Range"), // type
        tree!(5, "findRangeBounds"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "findRangeBounds"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "dataset"), // identifier
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the range of values the renderer requires to display all the\\n     * items from the specified dataset.\\n     *\\n     * @param dataset  the dataset (<code>null</code> permitted).\\n     * @param includeInterval  include the y-interval if the dataset has one.\\n     *\\n     * @return The range (<code>null</code> if the dataset is <code>null</code>\\n     *         or empty).\\n     *\\n     * @since 1.0.13\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "Range"), // type
        tree!(5, "findRangeBounds"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "includeInterval"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "dataset"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "getDataBoundsIncludesVisibleSeriesOnly"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "List"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "visibleSeriesKeys"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "ArrayList"), // type
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "int"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "seriesCount"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getRowCount"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(57; [ // for_statement
                tree!(50; [ // local_variable_declaration
                  tree!(14, "int"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "s"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(24, "0"), // decimal_integer_literal
                  ]),
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "s"), // identifier
                  tree!(43, "<"), // comparison_operator
                  tree!(5, "seriesCount"), // identifier
                ]),
                tree!(58; [ // update_expression
                  tree!(5, "s"), // identifier
                  tree!(59, "++"), // increment_operator
                ]),
                tree!(37; [ // block
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(52; [ // method_invocation
                        tree!(5, "isSeriesVisible"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "s"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(29; [ // expression_statement
                        tree!(52; [ // method_invocation
                          tree!(5, "visibleSeriesKeys"), // identifier
                          tree!(5, "add"), // identifier
                          tree!(35; [ // argument_list
                            tree!(52; [ // method_invocation
                              tree!(5, "dataset"), // identifier
                              tree!(5, "getRowKey"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "s"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(38; [ // return_statement
                tree!(52; [ // method_invocation
                  tree!(5, "DatasetUtilities"), // identifier
                  tree!(5, "findRangeBounds"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "dataset"), // identifier
                    tree!(5, "visibleSeriesKeys"), // identifier
                    tree!(5, "includeInterval"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(52; [ // method_invocation
                  tree!(5, "DatasetUtilities"), // identifier
                  tree!(5, "findRangeBounds"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "dataset"), // identifier
                    tree!(5, "includeInterval"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the Java2D coordinate for the middle of the specified data item.\\n     *\\n     * @param rowKey  the row key.\\n     * @param columnKey  the column key.\\n     * @param dataset  the dataset.\\n     * @param axis  the axis.\\n     * @param area  the data area.\\n     * @param edge  the edge along which the axis lies.\\n     *\\n     * @return The Java2D coordinate for the middle of the item.\\n     *\\n     * @since 1.0.11\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(65; [ // floating_point_type
          tree!(66, "double"), // double
        ]),
        tree!(5, "getItemMiddle"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "rowKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "columnKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "area"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleEdge"), // type
            tree!(5, "edge"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "axis"), // identifier
              tree!(5, "getCategoryMiddle"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "columnKey"), // identifier
                tree!(52; [ // method_invocation
                  tree!(5, "dataset"), // identifier
                  tree!(5, "getColumnKeys"), // identifier
                  tree!(35), // argument_list
                ]),
                tree!(5, "area"), // identifier
                tree!(5, "edge"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a background for the data area.  The default implementation just\\n     * gets the plot to draw the background, but some renderers will override\\n     * this behaviour.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param dataArea  the data area.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawBackground"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "plot"), // identifier
              tree!(5, "drawBackground"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "g2"), // identifier
                tree!(5, "dataArea"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws an outline for the data area.  The default implementation just\\n     * gets the plot to draw the outline, but some renderers will override this\\n     * behaviour.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param dataArea  the data area.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawOutline"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "plot"), // identifier
              tree!(5, "drawOutline"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "g2"), // identifier
                tree!(5, "dataArea"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a grid line against the domain axis.\\n     * <P>\\n     * Note that this default implementation assumes that the horizontal axis\\n     * is the domain axis. If this is not the case, you will need to override\\n     * this method.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param dataArea  the area for plotting data (not yet adjusted for any\\n     *                  3D effect).\\n     * @param value  the Java2D value at which the grid line should be drawn.\\n     * @param paint  the paint (<code>null</code> not permitted).\\n     * @param stroke  the stroke (<code>null</code> not permitted).\\n     *\\n     * @see #drawRangeGridline(Graphics2D, CategoryPlot, ValueAxis,\\n     *     Rectangle2D, double)\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawDomainLine"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "value"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Paint"), // type
            tree!(5, "paint"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Stroke"), // type
            tree!(5, "stroke"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "paint"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'paint' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "stroke"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'stroke' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Line2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "line"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "PlotOrientation"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "orientation"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getOrientation"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "Line2D.Double"), // type
                    tree!(35; [ // argument_list
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMinX"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "value"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMaxX"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "value"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "line"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "Line2D.Double"), // type
                      tree!(35; [ // argument_list
                        tree!(5, "value"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMinY"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(5, "value"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMaxY"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setPaint"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "paint"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setStroke"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "stroke"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "draw"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "line"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a line perpendicular to the range axis.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param axis  the value axis.\\n     * @param dataArea  the area for plotting data (not yet adjusted for any 3D\\n     *                  effect).\\n     * @param value  the value at which the grid line should be drawn.\\n     * @param paint  the paint (<code>null</code> not permitted).\\n     * @param stroke  the stroke (<code>null</code> not permitted).\\n     *\\n     * @see #drawRangeGridline\\n     *\\n     * @since 1.0.13\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawRangeLine"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "value"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Paint"), // type
            tree!(5, "paint"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Stroke"), // type
            tree!(5, "stroke"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Range"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "range"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "axis"), // identifier
                tree!(5, "getRange"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "range"), // identifier
                  tree!(5, "contains"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "value"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38), // return_statement
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "PlotOrientation"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "orientation"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getOrientation"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Line2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "line"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(22; [ // variable_declarator
              tree!(5, "v"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "axis"), // identifier
                tree!(5, "valueToJava2D"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "value"), // identifier
                  tree!(5, "dataArea"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getRangeAxisEdge"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "Line2D.Double"), // type
                    tree!(35; [ // argument_list
                      tree!(5, "v"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMinY"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "v"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMaxY"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "line"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "Line2D.Double"), // type
                      tree!(35; [ // argument_list
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMinX"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(5, "v"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMaxX"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(5, "v"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setPaint"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "paint"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setStroke"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "stroke"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "draw"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "line"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a marker for the domain axis.\\n     *\\n     * @param g2  the graphics device (not <code>null</code>).\\n     * @param plot  the plot (not <code>null</code>).\\n     * @param axis  the range axis (not <code>null</code>).\\n     * @param marker  the marker to be drawn (not <code>null</code>).\\n     * @param dataArea  the area inside the axes (not <code>null</code>).\\n     *\\n     * @see #drawRangeMarker(Graphics2D, CategoryPlot, ValueAxis, Marker,\\n     *     Rectangle2D)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawDomainMarker"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryMarker"), // type
            tree!(5, "marker"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Comparable"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "category"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getKey"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDataset"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "dataset"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getDataset"), // identifier
                tree!(35; [ // argument_list
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getIndexOf"), // identifier
                    tree!(35; [ // argument_list
                      tree!(32, "this"), // this
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "columnIndex"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getColumnIndex"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "category"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "columnIndex"), // identifier
                tree!(43, "<"), // comparison_operator
                tree!(24, "0"), // decimal_integer_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38), // return_statement
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(8; [ // modifiers
              tree!(21, "final"), // final
            ]),
            tree!(14, "Composite"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "savedComposite"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "g2"), // identifier
                tree!(5, "getComposite"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setComposite"), // identifier
              tree!(35; [ // argument_list
                tree!(52; [ // method_invocation
                  tree!(5, "AlphaComposite"), // identifier
                  tree!(5, "getInstance"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(5, "AlphaComposite"), // identifier
                      tree!(5, "SRC_OVER"), // identifier
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getAlpha"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "PlotOrientation"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "orientation"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getOrientation"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "bounds"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getDrawAsLine"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getCategoryMiddle"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "columnIndex"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataset"), // identifier
                        tree!(5, "getColumnCount"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getDomainAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Line2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "orientation"), // identifier
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "HORIZONTAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "line"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Line2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMinX"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMaxX"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "VERTICAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "line"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Line2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(5, "v"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "v"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setStroke"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getStroke"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "draw"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "line"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "bounds"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "line"), // identifier
                    tree!(5, "getBounds2D"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v0"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getCategoryStart"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "columnIndex"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataset"), // identifier
                        tree!(5, "getColumnCount"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getDomainAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v1"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getCategoryEnd"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "columnIndex"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataset"), // identifier
                        tree!(5, "getColumnCount"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getDomainAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Rectangle2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "area"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "orientation"), // identifier
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "HORIZONTAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "area"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Rectangle2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMinX"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v0"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getWidth"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(41; [ // parenthesized_expression
                            tree!(42; [ // binary_expression
                              tree!(5, "v1"), // identifier
                              tree!(69, "-"), // arithmetic_operator
                              tree!(5, "v0"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "VERTICAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "area"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Rectangle2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(5, "v0"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(41; [ // parenthesized_expression
                              tree!(42; [ // binary_expression
                                tree!(5, "v1"), // identifier
                                tree!(69, "-"), // arithmetic_operator
                                tree!(5, "v0"), // identifier
                              ]),
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getHeight"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "fill"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "area"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "bounds"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(5, "area"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "label"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getLabel"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "RectangleAnchor"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "anchor"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getLabelAnchor"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "label"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "Font"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "labelFont"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "marker"), // identifier
                    tree!(5, "getLabelFont"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setFont"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "labelFont"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabelPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Point2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "coordinates"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "calculateDomainMarkerTextAnchorPoint"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "g2"), // identifier
                      tree!(5, "orientation"), // identifier
                      tree!(5, "dataArea"), // identifier
                      tree!(5, "bounds"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "marker"), // identifier
                        tree!(5, "getLabelOffset"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "marker"), // identifier
                        tree!(5, "getLabelOffsetType"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "anchor"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "TextUtilities"), // identifier
                  tree!(5, "drawAlignedString"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "label"), // identifier
                    tree!(5, "g2"), // identifier
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "coordinates"), // identifier
                        tree!(5, "getX"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "coordinates"), // identifier
                        tree!(5, "getY"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabelTextAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setComposite"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "savedComposite"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a marker for the range axis.\\n     *\\n     * @param g2  the graphics device (not <code>null</code>).\\n     * @param plot  the plot (not <code>null</code>).\\n     * @param axis  the range axis (not <code>null</code>).\\n     * @param marker  the marker to be drawn (not <code>null</code>).\\n     * @param dataArea  the area inside the axes (not <code>null</code>).\\n     *\\n     * @see #drawDomainMarker(Graphics2D, CategoryPlot, CategoryAxis,\\n     *     CategoryMarker, Rectangle2D)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawRangeMarker"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Marker"), // type
            tree!(5, "marker"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(5, "marker"), // identifier
                tree!(62, "instanceof"), // instanceof
                tree!(14, "ValueMarker"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "ValueMarker"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "vm"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ValueMarker"), // type
                    tree!(5, "marker"), // identifier
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "value"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "vm"), // identifier
                    tree!(5, "getValue"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Range"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "range"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getRange"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(67; [ // unary_expression
                    tree!(68, "!"), // !
                    tree!(52; [ // method_invocation
                      tree!(5, "range"), // identifier
                      tree!(5, "contains"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "value"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(38), // return_statement
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(8; [ // modifiers
                  tree!(21, "final"), // final
                ]),
                tree!(14, "Composite"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "savedComposite"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "getComposite"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setComposite"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "AlphaComposite"), // identifier
                      tree!(5, "getInstance"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(5, "AlphaComposite"), // identifier
                          tree!(5, "SRC_OVER"), // identifier
                        ]),
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getAlpha"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "PlotOrientation"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "orientation"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getOrientation"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "valueToJava2D"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "value"), // identifier
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getRangeAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Line2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "orientation"), // identifier
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "HORIZONTAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "line"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Line2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(5, "v"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMinY"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMaxY"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "VERTICAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "line"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Line2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinX"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "v"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxX"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "v"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setStroke"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getStroke"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "draw"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "line"), // identifier
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "String"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "label"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "marker"), // identifier
                    tree!(5, "getLabel"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "RectangleAnchor"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "anchor"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "marker"), // identifier
                    tree!(5, "getLabelAnchor"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "label"), // identifier
                    tree!(43, "!="), // comparison_operator
                    tree!(44, "null"), // null_literal
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "Font"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "labelFont"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "marker"), // identifier
                        tree!(5, "getLabelFont"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "g2"), // identifier
                      tree!(5, "setFont"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "labelFont"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "g2"), // identifier
                      tree!(5, "setPaint"), // identifier
                      tree!(35; [ // argument_list
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getLabelPaint"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "Point2D"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "coordinates"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "calculateRangeMarkerTextAnchorPoint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "g2"), // identifier
                          tree!(5, "orientation"), // identifier
                          tree!(5, "dataArea"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "line"), // identifier
                            tree!(5, "getBounds2D"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getLabelOffset"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(31; [ // field_access
                            tree!(5, "LengthAdjustmentType"), // identifier
                            tree!(5, "EXPAND"), // identifier
                          ]),
                          tree!(5, "anchor"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "TextUtilities"), // identifier
                      tree!(5, "drawAlignedString"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "label"), // identifier
                        tree!(5, "g2"), // identifier
                        tree!(51; [ // cast_expression
                          tree!(65; [ // floating_point_type
                            tree!(70, "float"), // float
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "coordinates"), // identifier
                            tree!(5, "getX"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                        tree!(51; [ // cast_expression
                          tree!(65; [ // floating_point_type
                            tree!(70, "float"), // float
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "coordinates"), // identifier
                            tree!(5, "getY"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getLabelTextAnchor"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setComposite"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "savedComposite"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(61; [ // instanceof_expression
                  tree!(5, "marker"), // identifier
                  tree!(62, "instanceof"), // instanceof
                  tree!(14, "IntervalMarker"), // type
                ]),
              ]),
              tree!(37; [ // block
                tree!(50; [ // local_variable_declaration
                  tree!(14, "IntervalMarker"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "im"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(51; [ // cast_expression
                      tree!(14, "IntervalMarker"), // type
                      tree!(5, "marker"), // identifier
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "start"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "im"), // identifier
                      tree!(5, "getStartValue"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "end"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "im"), // identifier
                      tree!(5, "getEndValue"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "Range"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "range"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "axis"), // identifier
                      tree!(5, "getRange"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(67; [ // unary_expression
                      tree!(68, "!"), // !
                      tree!(41; [ // parenthesized_expression
                        tree!(52; [ // method_invocation
                          tree!(5, "range"), // identifier
                          tree!(5, "intersects"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "start"), // identifier
                            tree!(5, "end"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(38), // return_statement
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(8; [ // modifiers
                    tree!(21, "final"), // final
                  ]),
                  tree!(14, "Composite"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "savedComposite"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "g2"), // identifier
                      tree!(5, "getComposite"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "setComposite"), // identifier
                    tree!(35; [ // argument_list
                      tree!(52; [ // method_invocation
                        tree!(5, "AlphaComposite"), // identifier
                        tree!(5, "getInstance"), // identifier
                        tree!(35; [ // argument_list
                          tree!(31; [ // field_access
                            tree!(5, "AlphaComposite"), // identifier
                            tree!(5, "SRC_OVER"), // identifier
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getAlpha"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "start2d"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "axis"), // identifier
                      tree!(5, "valueToJava2D"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "start"), // identifier
                        tree!(5, "dataArea"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "plot"), // identifier
                          tree!(5, "getRangeAxisEdge"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "end2d"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "axis"), // identifier
                      tree!(5, "valueToJava2D"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "end"), // identifier
                        tree!(5, "dataArea"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "plot"), // identifier
                          tree!(5, "getRangeAxisEdge"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "low"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "Math"), // identifier
                      tree!(5, "min"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "start2d"), // identifier
                        tree!(5, "end2d"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "high"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "Math"), // identifier
                      tree!(5, "max"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "start2d"), // identifier
                        tree!(5, "end2d"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "PlotOrientation"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "orientation"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "plot"), // identifier
                      tree!(5, "getOrientation"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "Rectangle2D"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "rect"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(44, "null"), // null_literal
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "HORIZONTAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(49, "// clip left and right bounds to data area"), // line_comment
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "low"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "Math"), // identifier
                          tree!(5, "max"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "low"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "high"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "Math"), // identifier
                          tree!(5, "min"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "high"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "rect"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Rectangle2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(5, "low"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(42; [ // binary_expression
                              tree!(5, "high"), // identifier
                              tree!(69, "-"), // arithmetic_operator
                              tree!(5, "low"), // identifier
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getHeight"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(42; [ // binary_expression
                        tree!(5, "orientation"), // identifier
                        tree!(43, "=="), // comparison_operator
                        tree!(31; [ // field_access
                          tree!(5, "PlotOrientation"), // identifier
                          tree!(5, "VERTICAL"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(49, "// clip top and bottom bounds to data area"), // line_comment
                      tree!(29; [ // expression_statement
                        tree!(30; [ // assignment_expression
                          tree!(5, "low"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "Math"), // identifier
                            tree!(5, "max"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "low"), // identifier
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getMinY"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(29; [ // expression_statement
                        tree!(30; [ // assignment_expression
                          tree!(5, "high"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "Math"), // identifier
                            tree!(5, "min"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "high"), // identifier
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getMaxY"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(29; [ // expression_statement
                        tree!(30; [ // assignment_expression
                          tree!(5, "rect"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(33; [ // object_creation_expression
                            tree!(34, "new"), // new
                            tree!(14, "Rectangle2D.Double"), // type
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getMinX"), // identifier
                                tree!(35), // argument_list
                              ]),
                              tree!(5, "low"), // identifier
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getWidth"), // identifier
                                tree!(35), // argument_list
                              ]),
                              tree!(42; [ // binary_expression
                                tree!(5, "high"), // identifier
                                tree!(69, "-"), // arithmetic_operator
                                tree!(5, "low"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "Paint"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "p"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(61; [ // instanceof_expression
                      tree!(5, "p"), // identifier
                      tree!(62, "instanceof"), // instanceof
                      tree!(14, "GradientPaint"), // type
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "GradientPaint"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "gp"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(51; [ // cast_expression
                          tree!(14, "GradientPaint"), // type
                          tree!(5, "p"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "GradientPaintTransformer"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "t"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "im"), // identifier
                          tree!(5, "getGradientPaintTransformer"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                    tree!(40; [ // if_statement
                      tree!(41; [ // parenthesized_expression
                        tree!(42; [ // binary_expression
                          tree!(5, "t"), // identifier
                          tree!(43, "!="), // comparison_operator
                          tree!(44, "null"), // null_literal
                        ]),
                      ]),
                      tree!(37; [ // block
                        tree!(29; [ // expression_statement
                          tree!(30; [ // assignment_expression
                            tree!(5, "gp"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "t"), // identifier
                              tree!(5, "transform"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "gp"), // identifier
                                tree!(5, "rect"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setPaint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "gp"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setPaint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "p"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "fill"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "rect"), // identifier
                    ]),
                  ]),
                ]),
                tree!(49, "// now draw the outlines, if visible..."), // line_comment
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(42; [ // binary_expression
                        tree!(52; [ // method_invocation
                          tree!(5, "im"), // identifier
                          tree!(5, "getOutlinePaint"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(43, "!="), // comparison_operator
                        tree!(44, "null"), // null_literal
                      ]),
                      tree!(63, "&&"), // logical_operator
                      tree!(42; [ // binary_expression
                        tree!(52; [ // method_invocation
                          tree!(5, "im"), // identifier
                          tree!(5, "getOutlineStroke"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(43, "!="), // comparison_operator
                        tree!(44, "null"), // null_literal
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(40; [ // if_statement
                      tree!(41; [ // parenthesized_expression
                        tree!(42; [ // binary_expression
                          tree!(5, "orientation"), // identifier
                          tree!(43, "=="), // comparison_operator
                          tree!(31; [ // field_access
                            tree!(5, "PlotOrientation"), // identifier
                            tree!(5, "VERTICAL"), // identifier
                          ]),
                        ]),
                      ]),
                      tree!(37; [ // block
                        tree!(50; [ // local_variable_declaration
                          tree!(14, "Line2D"), // type
                          tree!(22; [ // variable_declarator
                            tree!(5, "line"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(14, "Line2D.Double"), // type
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "x0"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "x1"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setPaint"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlinePaint"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setStroke"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlineStroke"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "start"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "x0"), // identifier
                                  tree!(5, "start2d"), // identifier
                                  tree!(5, "x1"), // identifier
                                  tree!(5, "start2d"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "end"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "x0"), // identifier
                                  tree!(5, "end2d"), // identifier
                                  tree!(5, "x1"), // identifier
                                  tree!(5, "end2d"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(37; [ // block
                        tree!(49, "// PlotOrientation.HORIZONTAL"), // line_comment
                        tree!(50; [ // local_variable_declaration
                          tree!(14, "Line2D"), // type
                          tree!(22; [ // variable_declarator
                            tree!(5, "line"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(14, "Line2D.Double"), // type
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "y0"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "y1"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setPaint"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlinePaint"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setStroke"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlineStroke"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "start"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "start2d"), // identifier
                                  tree!(5, "y0"), // identifier
                                  tree!(5, "start2d"), // identifier
                                  tree!(5, "y1"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "end"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "end2d"), // identifier
                                  tree!(5, "y0"), // identifier
                                  tree!(5, "end2d"), // identifier
                                  tree!(5, "y1"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "String"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "label"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabel"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "RectangleAnchor"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "anchor"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabelAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "label"), // identifier
                      tree!(43, "!="), // comparison_operator
                      tree!(44, "null"), // null_literal
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "Font"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "labelFont"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getLabelFont"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setFont"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "labelFont"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setPaint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getLabelPaint"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "Point2D"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "coordinates"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "calculateRangeMarkerTextAnchorPoint"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "g2"), // identifier
                            tree!(5, "orientation"), // identifier
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "rect"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "marker"), // identifier
                              tree!(5, "getLabelOffset"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "marker"), // identifier
                              tree!(5, "getLabelOffsetType"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "anchor"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "TextUtilities"), // identifier
                        tree!(5, "drawAlignedString"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "label"), // identifier
                          tree!(5, "g2"), // identifier
                          tree!(51; [ // cast_expression
                            tree!(65; [ // floating_point_type
                              tree!(70, "float"), // float
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "coordinates"), // identifier
                              tree!(5, "getX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                          tree!(51; [ // cast_expression
                            tree!(65; [ // floating_point_type
                              tree!(70, "float"), // float
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "coordinates"), // identifier
                              tree!(5, "getY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getLabelTextAnchor"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "setComposite"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "savedComposite"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Calculates the (x, y) coordinates for drawing the label for a marker on\\n     * the range axis.\\n     *\\n     * @param g2  the graphics device.\\n     * @param orientation  the plot orientation.\\n     * @param dataArea  the data area.\\n     * @param markerArea  the rectangle surrounding the marker.\\n     * @param markerOffset  the marker offset.\\n     * @param labelOffsetType  the label offset type.\\n     * @param anchor  the label anchor.\\n     *\\n     * @return The coordinates for drawing the marker label.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "Point2D"), // type
        tree!(5, "calculateDomainMarkerTextAnchorPoint"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "markerArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleInsets"), // type
            tree!(5, "markerOffset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "LengthAdjustmentType"), // type
            tree!(5, "labelOffsetType"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleAnchor"), // type
            tree!(5, "anchor"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "anchorRect"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "anchorRect"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "markerOffset"), // identifier
                    tree!(5, "createAdjustedRectangle"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "markerArea"), // identifier
                      tree!(31; [ // field_access
                        tree!(5, "LengthAdjustmentType"), // identifier
                        tree!(5, "CONTRACT"), // identifier
                      ]),
                      tree!(5, "labelOffsetType"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "anchorRect"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "markerOffset"), // identifier
                      tree!(5, "createAdjustedRectangle"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "markerArea"), // identifier
                        tree!(5, "labelOffsetType"), // identifier
                        tree!(31; [ // field_access
                          tree!(5, "LengthAdjustmentType"), // identifier
                          tree!(5, "CONTRACT"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "RectangleAnchor"), // identifier
              tree!(5, "coordinates"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "anchorRect"), // identifier
                tree!(5, "anchor"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Calculates the (x, y) coordinates for drawing a marker label.\\n     *\\n     * @param g2  the graphics device.\\n     * @param orientation  the plot orientation.\\n     * @param dataArea  the data area.\\n     * @param markerArea  the rectangle surrounding the marker.\\n     * @param markerOffset  the marker offset.\\n     * @param labelOffsetType  the label offset type.\\n     * @param anchor  the label anchor.\\n     *\\n     * @return The coordinates for drawing the marker label.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "Point2D"), // type
        tree!(5, "calculateRangeMarkerTextAnchorPoint"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "markerArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleInsets"), // type
            tree!(5, "markerOffset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "LengthAdjustmentType"), // type
            tree!(5, "labelOffsetType"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleAnchor"), // type
            tree!(5, "anchor"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "anchorRect"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "anchorRect"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "markerOffset"), // identifier
                    tree!(5, "createAdjustedRectangle"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "markerArea"), // identifier
                      tree!(5, "labelOffsetType"), // identifier
                      tree!(31; [ // field_access
                        tree!(5, "LengthAdjustmentType"), // identifier
                        tree!(5, "CONTRACT"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "anchorRect"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "markerOffset"), // identifier
                      tree!(5, "createAdjustedRectangle"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "markerArea"), // identifier
                        tree!(31; [ // field_access
                          tree!(5, "LengthAdjustmentType"), // identifier
                          tree!(5, "CONTRACT"), // identifier
                        ]),
                        tree!(5, "labelOffsetType"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "RectangleAnchor"), // identifier
              tree!(5, "coordinates"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "anchorRect"), // identifier
                tree!(5, "anchor"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a legend item for a series.  This default implementation will\\n     * return <code>null</code> if {@link #isSeriesVisible(int)} or\\n     * {@link #isSeriesVisibleInLegend(int)} returns <code>false</code>.\\n     *\\n     * @param datasetIndex  the dataset index (zero-based).\\n     * @param series  the series index (zero-based).\\n     *\\n     * @return The legend item (possibly <code>null</code>).\\n     *\\n     * @see #getLegendItems()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "LegendItem"), // type
        tree!(5, "getLegendItem"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "datasetIndex"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryPlot"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "p"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getPlot"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "p"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(49, "// check that a legend item needs to be displayed..."), // line_comment
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(67; [ // unary_expression
                  tree!(68, "!"), // !
                  tree!(52; [ // method_invocation
                    tree!(5, "isSeriesVisible"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
                tree!(63, "||"), // logical_operator
                tree!(67; [ // unary_expression
                  tree!(68, "!"), // !
                  tree!(52; [ // method_invocation
                    tree!(5, "isSeriesVisibleInLegend"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDataset"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "dataset"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "p"), // identifier
                tree!(5, "getDataset"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "datasetIndex"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "label"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemLabelGenerator"), // identifier
                ]),
                tree!(5, "generateLabel"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "dataset"), // identifier
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "description"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(5, "label"), // identifier
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "toolTipText"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemToolTipGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "toolTipText"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemToolTipGenerator"), // identifier
                    ]),
                    tree!(5, "generateLabel"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "urlText"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemURLGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "urlText"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemURLGenerator"), // identifier
                    ]),
                    tree!(5, "generateLabel"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Shape"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "shape"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupLegendShape"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Paint"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "paint"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupSeriesPaint"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Paint"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "outlinePaint"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupSeriesOutlinePaint"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Stroke"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "outlineStroke"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupSeriesOutlineStroke"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "LegendItem"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "item"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "LegendItem"), // type
                tree!(35; [ // argument_list
                  tree!(5, "label"), // identifier
                  tree!(5, "description"), // identifier
                  tree!(5, "toolTipText"), // identifier
                  tree!(5, "urlText"), // identifier
                  tree!(5, "shape"), // identifier
                  tree!(5, "paint"), // identifier
                  tree!(5, "outlineStroke"), // identifier
                  tree!(5, "outlinePaint"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setLabelFont"), // identifier
              tree!(35; [ // argument_list
                tree!(52; [ // method_invocation
                  tree!(5, "lookupLegendTextFont"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "series"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Paint"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "labelPaint"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupLegendTextPaint"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "labelPaint"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "item"), // identifier
                  tree!(5, "setLabelPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "labelPaint"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setSeriesKey"), // identifier
              tree!(35; [ // argument_list
                tree!(52; [ // method_invocation
                  tree!(5, "dataset"), // identifier
                  tree!(5, "getRowKey"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "series"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setSeriesIndex"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setDataset"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "dataset"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setDatasetIndex"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "datasetIndex"), // identifier
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "item"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Tests this renderer for equality with another object.\\n     *\\n     * @param obj  the object.\\n     *\\n     * @return <code>true</code> or <code>false</code>.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "boolean"), // type
        tree!(5, "equals"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Object"), // type
            tree!(5, "obj"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "obj"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(32, "this"), // this
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(5, "obj"), // identifier
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "AbstractCategoryItemRenderer"), // type
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "AbstractCategoryItemRenderer"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "that"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "AbstractCategoryItemRenderer"), // type
                tree!(5, "obj"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "itemLabelGeneratorList"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "itemLabelGeneratorList"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseItemLabelGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "baseItemLabelGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "toolTipGeneratorList"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "toolTipGeneratorList"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseToolTipGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "baseToolTipGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "urlGeneratorList"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "urlGeneratorList"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseURLGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "baseURLGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemLabelGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "legendItemLabelGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemToolTipGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "legendItemToolTipGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemURLGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "legendItemURLGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "backgroundAnnotations"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "backgroundAnnotations"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "foregroundAnnotations"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "foregroundAnnotations"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(71, "super"), // super
              tree!(5, "equals"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "obj"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a hash code for the renderer.\\n     *\\n     * @return The hash code.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "hashCode"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(71, "super"), // super
                tree!(5, "hashCode"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the drawing supplier from the plot.\\n     *\\n     * @return The drawing supplier (possibly <code>null</code>).\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "DrawingSupplier"), // type
        tree!(5, "getDrawingSupplier"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "DrawingSupplier"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryPlot"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "cp"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getPlot"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "cp"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "cp"), // identifier
                    tree!(5, "getDrawingSupplier"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Considers the current (x, y) coordinate and updates the crosshair point\\n     * if it meets the criteria (usually means the (x, y) coordinate is the\\n     * closest to the anchor point so far).\\n     *\\n     * @param crosshairState  the crosshair state (<code>null</code> permitted,\\n     *                        but the method does nothing in that case).\\n     * @param rowKey  the row key.\\n     * @param columnKey  the column key.\\n     * @param value  the data value.\\n     * @param datasetIndex  the dataset index.\\n     * @param transX  the x-value translated to Java2D space.\\n     * @param transY  the y-value translated to Java2D space.\\n     * @param orientation  the plot orientation (<code>null</code> not\\n     *                     permitted).\\n     *\\n     * @since 1.0.11\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "updateCrosshairValues"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryCrosshairState"), // type
            tree!(5, "crosshairState"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "rowKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "columnKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "value"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "datasetIndex"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "transX"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "transY"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'orientation' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "crosshairState"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "plot"), // identifier
                    ]),
                    tree!(5, "isRangeCrosshairLockedOnData"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(49, "// both axes"), // line_comment
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "crosshairState"), // identifier
                      tree!(5, "updateCrosshairPoint"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "rowKey"), // identifier
                        tree!(5, "columnKey"), // identifier
                        tree!(5, "value"), // identifier
                        tree!(5, "datasetIndex"), // identifier
                        tree!(5, "transX"), // identifier
                        tree!(5, "transY"), // identifier
                        tree!(5, "orientation"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "crosshairState"), // identifier
                      tree!(5, "updateCrosshairX"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "rowKey"), // identifier
                        tree!(5, "columnKey"), // identifier
                        tree!(5, "datasetIndex"), // identifier
                        tree!(5, "transX"), // identifier
                        tree!(5, "orientation"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws an item label.\\n     *\\n     * @param g2  the graphics device.\\n     * @param orientation  the orientation.\\n     * @param dataset  the dataset.\\n     * @param row  the row.\\n     * @param column  the column.\\n     * @param selected  is the item selected?\\n     * @param x  the x coordinate (in Java2D space).\\n     * @param y  the y coordinate (in Java2D space).\\n     * @param negative  indicates a negative value (which affects the item\\n     *                  label position).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawItemLabel"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "x"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "y"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "negative"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getItemLabelGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "Font"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "labelFont"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "getItemLabelFont"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                      tree!(5, "selected"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Paint"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "paint"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "getItemLabelPaint"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                      tree!(5, "selected"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setFont"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "labelFont"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "paint"), // identifier
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "String"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "label"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "generator"), // identifier
                    tree!(5, "generateLabel"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "ItemLabelPosition"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "position"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(67; [ // unary_expression
                    tree!(68, "!"), // !
                    tree!(5, "negative"), // identifier
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "position"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "getPositiveItemLabelPosition"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "row"), // identifier
                          tree!(5, "column"), // identifier
                          tree!(5, "selected"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "position"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "getNegativeItemLabelPosition"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "row"), // identifier
                          tree!(5, "column"), // identifier
                          tree!(5, "selected"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Point2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "anchorPoint"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "calculateLabelAnchorPoint"), // identifier
                    tree!(35; [ // argument_list
                      tree!(52; [ // method_invocation
                        tree!(5, "position"), // identifier
                        tree!(5, "getItemLabelAnchor"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "x"), // identifier
                      tree!(5, "y"), // identifier
                      tree!(5, "orientation"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "TextUtilities"), // identifier
                  tree!(5, "drawRotatedString"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "label"), // identifier
                    tree!(5, "g2"), // identifier
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "anchorPoint"), // identifier
                        tree!(5, "getX"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "anchorPoint"), // identifier
                        tree!(5, "getY"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "position"), // identifier
                      tree!(5, "getTextAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "position"), // identifier
                      tree!(5, "getAngle"), // identifier
                      tree!(35), // argument_list
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "position"), // identifier
                      tree!(5, "getRotationAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws all the annotations for the specified layer.\\n     *\\n     * @param g2  the graphics device.\\n     * @param dataArea  the data area.\\n     * @param domainAxis  the domain axis.\\n     * @param rangeAxis  the range axis.\\n     * @param layer  the layer.\\n     * @param info  the plot rendering info.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawAnnotations"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Layer"), // type
            tree!(5, "layer"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotRenderingInfo"), // type
            tree!(5, "info"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Iterator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "iterator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "layer"), // identifier
                tree!(5, "equals"), // identifier
                tree!(35; [ // argument_list
                  tree!(31; [ // field_access
                    tree!(5, "Layer"), // identifier
                    tree!(5, "FOREGROUND"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "iterator"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "foregroundAnnotations"), // identifier
                    ]),
                    tree!(5, "iterator"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(52; [ // method_invocation
                  tree!(5, "layer"), // identifier
                  tree!(5, "equals"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(5, "Layer"), // identifier
                      tree!(5, "BACKGROUND"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "iterator"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "backgroundAnnotations"), // identifier
                      ]),
                      tree!(5, "iterator"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(49, "// should not get here"), // line_comment
                tree!(45; [ // throw_statement
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "RuntimeException"), // type
                    tree!(35; [ // argument_list
                      tree!(46; [ // string_literal
                        tree!(47, "\""), // "
                        tree!(48, "Unknown layer."), // string_fragment
                        tree!(47, "\""), // "
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(72; [ // while_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "iterator"), // identifier
                tree!(5, "hasNext"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "CategoryAnnotation"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "annotation"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategoryAnnotation"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "iterator"), // identifier
                      tree!(5, "next"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "annotation"), // identifier
                  tree!(5, "draw"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "g2"), // identifier
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "plot"), // identifier
                    ]),
                    tree!(5, "dataArea"), // identifier
                    tree!(5, "domainAxis"), // identifier
                    tree!(5, "rangeAxis"), // identifier
                    tree!(24, "0"), // decimal_integer_literal
                    tree!(5, "info"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns an independent copy of the renderer.  The <code>plot</code>\\n     * reference is shallow copied.\\n     *\\n     * @return A clone.\\n     *\\n     * @throws CloneNotSupportedException  can be thrown if one of the objects\\n     *         belonging to the renderer does not support cloning (for example,\\n     *         an item label generator).\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Object"), // type
        tree!(5, "clone"), // identifier
        tree!(27), // formal_parameters
        tree!(73; [ // throws
          tree!(73, "throws"), // throws
          tree!(14, "CloneNotSupportedException"), // type
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "AbstractCategoryItemRenderer"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "clone"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "AbstractCategoryItemRenderer"), // type
                tree!(52; [ // method_invocation
                  tree!(71, "super"), // super
                  tree!(5, "clone"), // identifier
                  tree!(35), // argument_list
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "itemLabelGeneratorList"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "itemLabelGeneratorList"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ObjectList"), // type
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "itemLabelGeneratorList"), // identifier
                      ]),
                      tree!(5, "clone"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "baseItemLabelGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseItemLabelGenerator"), // identifier
                    ]),
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "PublicCloneable"), // type
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "PublicCloneable"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "pc"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "PublicCloneable"), // type
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "baseItemLabelGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(31; [ // field_access
                        tree!(5, "clone"), // identifier
                        tree!(5, "baseItemLabelGenerator"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryItemLabelGenerator"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "pc"), // identifier
                          tree!(5, "clone"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(45; [ // throw_statement
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "CloneNotSupportedException"), // type
                      tree!(35; [ // argument_list
                        tree!(46; [ // string_literal
                          tree!(47, "\""), // "
                          tree!(48, "ItemLabelGenerator not cloneable."), // string_fragment
                          tree!(47, "\""), // "
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "toolTipGeneratorList"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "toolTipGeneratorList"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ObjectList"), // type
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "toolTipGeneratorList"), // identifier
                      ]),
                      tree!(5, "clone"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "baseToolTipGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseToolTipGenerator"), // identifier
                    ]),
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "PublicCloneable"), // type
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "PublicCloneable"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "pc"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "PublicCloneable"), // type
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "baseToolTipGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(31; [ // field_access
                        tree!(5, "clone"), // identifier
                        tree!(5, "baseToolTipGenerator"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryToolTipGenerator"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "pc"), // identifier
                          tree!(5, "clone"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(45; [ // throw_statement
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "CloneNotSupportedException"), // type
                      tree!(35; [ // argument_list
                        tree!(46; [ // string_literal
                          tree!(47, "\""), // "
                          tree!(48, "Base tool tip generator not cloneable."), // string_fragment
                          tree!(47, "\""), // "
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "urlGeneratorList"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "urlGeneratorList"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ObjectList"), // type
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "urlGeneratorList"), // identifier
                      ]),
                      tree!(5, "clone"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "baseURLGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseURLGenerator"), // identifier
                    ]),
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "PublicCloneable"), // type
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "PublicCloneable"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "pc"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "PublicCloneable"), // type
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "baseURLGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(31; [ // field_access
                        tree!(5, "clone"), // identifier
                        tree!(5, "baseURLGenerator"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryURLGenerator"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "pc"), // identifier
                          tree!(5, "clone"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(45; [ // throw_statement
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "CloneNotSupportedException"), // type
                      tree!(35; [ // argument_list
                        tree!(46; [ // string_literal
                          tree!(47, "\""), // "
                          tree!(48, "Base item URL generator not cloneable."), // string_fragment
                          tree!(47, "\""), // "
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemLabelGenerator"), // identifier
                ]),
                tree!(62, "instanceof"), // instanceof
                tree!(14, "PublicCloneable"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "legendItemLabelGenerator"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategorySeriesLabelGenerator"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "ObjectUtilities"), // identifier
                      tree!(5, "clone"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "legendItemLabelGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemToolTipGenerator"), // identifier
                ]),
                tree!(62, "instanceof"), // instanceof
                tree!(14, "PublicCloneable"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "legendItemToolTipGenerator"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategorySeriesLabelGenerator"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "ObjectUtilities"), // identifier
                      tree!(5, "clone"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "legendItemToolTipGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemURLGenerator"), // identifier
                ]),
                tree!(62, "instanceof"), // instanceof
                tree!(14, "PublicCloneable"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "legendItemURLGenerator"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategorySeriesLabelGenerator"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "ObjectUtilities"), // identifier
                      tree!(5, "clone"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "legendItemURLGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "clone"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the domain axis that is used for the specified dataset.\\n     *\\n     * @param plot  the plot (<code>null</code> not permitted).\\n     * @param dataset  the dataset (<code>null</code> not permitted).\\n     *\\n     * @return A domain axis.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "CategoryAxis"), // type
        tree!(5, "getDomainAxis"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "datasetIndex"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "indexOf"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "dataset"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "plot"), // identifier
              tree!(5, "getDomainAxisForDataset"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "datasetIndex"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a range axis for a plot.\\n     *\\n     * @param plot  the plot.\\n     * @param index  the axis index.\\n     *\\n     * @return A range axis.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "ValueAxis"), // type
        tree!(5, "getRangeAxis"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "index"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "ValueAxis"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getRangeAxis"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "index"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "result"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getRangeAxis"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a (possibly empty) collection of legend items for the series\\n     * that this renderer is responsible for drawing.\\n     *\\n     * @return The legend item collection (never <code>null</code>).\\n     *\\n     * @see #getLegendItem(int, int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "LegendItemCollection"), // type
        tree!(5, "getLegendItems"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "LegendItemCollection"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "LegendItemCollection"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "plot"), // identifier
                ]),
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(5, "result"), // identifier
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "index"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "plot"), // identifier
                ]),
                tree!(5, "getIndexOf"), // identifier
                tree!(35; [ // argument_list
                  tree!(32, "this"), // this
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDataset"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "dataset"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "plot"), // identifier
                ]),
                tree!(5, "getDataset"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "index"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "dataset"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(5, "result"), // identifier
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "seriesCount"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getRowCount"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(52; [ // method_invocation
                  tree!(5, "plot"), // identifier
                  tree!(5, "getRowRenderingOrder"), // identifier
                  tree!(35), // argument_list
                ]),
                tree!(5, "equals"), // identifier
                tree!(35; [ // argument_list
                  tree!(31; [ // field_access
                    tree!(5, "SortOrder"), // identifier
                    tree!(5, "ASCENDING"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(57; [ // for_statement
                tree!(50; [ // local_variable_declaration
                  tree!(14, "int"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "i"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(24, "0"), // decimal_integer_literal
                  ]),
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "i"), // identifier
                  tree!(43, "<"), // comparison_operator
                  tree!(5, "seriesCount"), // identifier
                ]),
                tree!(58; [ // update_expression
                  tree!(5, "i"), // identifier
                  tree!(59, "++"), // increment_operator
                ]),
                tree!(37; [ // block
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(52; [ // method_invocation
                        tree!(5, "isSeriesVisibleInLegend"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "i"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(50; [ // local_variable_declaration
                        tree!(14, "LegendItem"), // type
                        tree!(22; [ // variable_declarator
                          tree!(5, "item"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "getLegendItem"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "index"), // identifier
                              tree!(5, "i"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(40; [ // if_statement
                        tree!(41; [ // parenthesized_expression
                          tree!(42; [ // binary_expression
                            tree!(5, "item"), // identifier
                            tree!(43, "!="), // comparison_operator
                            tree!(44, "null"), // null_literal
                          ]),
                        ]),
                        tree!(37; [ // block
                          tree!(29; [ // expression_statement
                            tree!(52; [ // method_invocation
                              tree!(5, "result"), // identifier
                              tree!(5, "add"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "item"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(57; [ // for_statement
                tree!(50; [ // local_variable_declaration
                  tree!(14, "int"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "i"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(42; [ // binary_expression
                      tree!(5, "seriesCount"), // identifier
                      tree!(69, "-"), // arithmetic_operator
                      tree!(24, "1"), // decimal_integer_literal
                    ]),
                  ]),
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "i"), // identifier
                  tree!(43, ">="), // comparison_operator
                  tree!(24, "0"), // decimal_integer_literal
                ]),
                tree!(58; [ // update_expression
                  tree!(5, "i"), // identifier
                  tree!(59, "--"), // increment_operator
                ]),
                tree!(37; [ // block
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(52; [ // method_invocation
                        tree!(5, "isSeriesVisibleInLegend"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "i"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(50; [ // local_variable_declaration
                        tree!(14, "LegendItem"), // type
                        tree!(22; [ // variable_declarator
                          tree!(5, "item"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "getLegendItem"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "index"), // identifier
                              tree!(5, "i"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(40; [ // if_statement
                        tree!(41; [ // parenthesized_expression
                          tree!(42; [ // binary_expression
                            tree!(5, "item"), // identifier
                            tree!(43, "!="), // comparison_operator
                            tree!(44, "null"), // null_literal
                          ]),
                        ]),
                        tree!(37; [ // block
                          tree!(29; [ // expression_statement
                            tree!(52; [ // method_invocation
                              tree!(5, "result"), // identifier
                              tree!(5, "add"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "item"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Adds an entity with the specified hotspot.\\n     *\\n     * @param entities  the entity collection.\\n     * @param hotspot  the hotspot (<code>null</code> not permitted).\\n     * @param dataset  the dataset.\\n     * @param row  the row index.\\n     * @param column  the column index.\\n     * @param selected  is the item selected?\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addEntity"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "EntityCollection"), // type
            tree!(5, "entities"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Shape"), // type
            tree!(5, "hotspot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "hotspot"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'hotspot' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "addEntity"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "entities"), // identifier
                tree!(5, "hotspot"), // identifier
                tree!(5, "dataset"), // identifier
                tree!(5, "row"), // identifier
                tree!(5, "column"), // identifier
                tree!(5, "selected"), // identifier
                tree!(74, "0.0"), // decimal_floating_point_literal
                tree!(74, "0.0"), // decimal_floating_point_literal
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Adds an entity to the collection.\\n     *\\n     * @param entities  the entity collection being populated.\\n     * @param hotspot  the entity area (if <code>null</code> a default will be\\n     *              used).\\n     * @param dataset  the dataset.\\n     * @param row  the series.\\n     * @param column  the item.\\n     * @param selected  is the item selected?\\n     * @param entityX  the entity's center x-coordinate in user space (only\\n     *                 used if <code>area</code> is <code>null</code>).\\n     * @param entityY  the entity's center y-coordinate in user space (only\\n     *                 used if <code>area</code> is <code>null</code>).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addEntity"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "EntityCollection"), // type
            tree!(5, "entities"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Shape"), // type
            tree!(5, "hotspot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "entityX"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "entityY"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "getItemCreateEntity"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "row"), // identifier
                    tree!(5, "column"), // identifier
                    tree!(5, "selected"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38), // return_statement
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Shape"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "s"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(5, "hotspot"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "hotspot"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "r"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "getDefaultEntityRadius"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "w"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(42; [ // binary_expression
                    tree!(5, "r"), // identifier
                    tree!(69, "*"), // arithmetic_operator
                    tree!(24, "2"), // decimal_integer_literal
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(52; [ // method_invocation
                      tree!(52; [ // method_invocation
                        tree!(5, "getPlot"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "getOrientation"), // identifier
                      tree!(35), // argument_list
                    ]),
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "VERTICAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "s"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Ellipse2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(42; [ // binary_expression
                            tree!(5, "entityX"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(42; [ // binary_expression
                            tree!(5, "entityY"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(5, "w"), // identifier
                          tree!(5, "w"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "s"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Ellipse2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(42; [ // binary_expression
                            tree!(5, "entityY"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(42; [ // binary_expression
                            tree!(5, "entityX"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(5, "w"), // identifier
                          tree!(5, "w"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "tip"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getToolTipGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "tip"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "generator"), // identifier
                    tree!(5, "generateToolTip"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "url"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryURLGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "urlster"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getURLGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "urlster"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "url"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "urlster"), // identifier
                    tree!(5, "generateURL"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemEntity"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "entity"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "CategoryItemEntity"), // type
                tree!(35; [ // argument_list
                  tree!(5, "s"), // identifier
                  tree!(5, "tip"), // identifier
                  tree!(5, "url"), // identifier
                  tree!(5, "dataset"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getRowKey"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                    ]),
                  ]),
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getColumnKey"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "entities"), // identifier
              tree!(5, "add"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "entity"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "        \\n     * Returns a shape that can be used for hit testing on a data item drawn\\n     * by the renderer.\\n     *\\n     * @param g2  the graphics device.\\n     * @param dataArea  the area within which the data is being rendered.\\n     * @param plot  the plot (can be used to obtain standard color\\n     *              information etc).\\n     * @param domainAxis  the domain axis.\\n     * @param rangeAxis  the range axis.\\n     * @param dataset  the dataset.\\n     * @param row  the row index (zero-based).\\n     * @param column  the column index (zero-based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return A shape equal to the hot spot for a data item.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Shape"), // type
        tree!(5, "createHotSpotShape"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemRendererState"), // type
            tree!(5, "state"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(45; [ // throw_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(14, "RuntimeException"), // type
              tree!(35; [ // argument_list
                tree!(46; [ // string_literal
                  tree!(47, "\""), // "
                  tree!(48, "Not implemented."), // string_fragment
                  tree!(47, "\""), // "
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the rectangular bounds for the hot spot for an item drawn by\\n     * this renderer.  This is intended to provide a quick test for\\n     * eliminating data points before more accurate testing against the\\n     * shape returned by createHotSpotShape().\\n     *\\n     * @param g2\\n     * @param dataArea\\n     * @param plot\\n     * @param domainAxis\\n     * @param rangeAxis\\n     * @param dataset\\n     * @param row\\n     * @param column\\n     * @param selected\\n     * @param result\\n     * @return\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Rectangle2D"), // type
        tree!(5, "createHotSpotBounds"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemRendererState"), // type
            tree!(5, "state"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "result"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "result"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "Rectangle"), // type
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Comparable"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "key"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getColumnKey"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "column"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Number"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "y"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getValue"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "y"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(22; [ // variable_declarator
              tree!(5, "xx"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "domainAxis"), // identifier
                tree!(5, "getCategoryMiddle"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "key"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getCategoriesForAxis"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "domainAxis"), // identifier
                    ]),
                  ]),
                  tree!(5, "dataArea"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getDomainAxisEdge"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(22; [ // variable_declarator
              tree!(5, "yy"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "rangeAxis"), // identifier
                tree!(5, "valueToJava2D"), // identifier
                tree!(35; [ // argument_list
                  tree!(52; [ // method_invocation
                    tree!(5, "y"), // identifier
                    tree!(5, "doubleValue"), // identifier
                    tree!(35), // argument_list
                  ]),
                  tree!(5, "dataArea"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getRangeAxisEdge"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "result"), // identifier
              tree!(5, "setRect"), // identifier
              tree!(35; [ // argument_list
                tree!(42; [ // binary_expression
                  tree!(5, "xx"), // identifier
                  tree!(69, "-"), // arithmetic_operator
                  tree!(24, "2"), // decimal_integer_literal
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "yy"), // identifier
                  tree!(69, "-"), // arithmetic_operator
                  tree!(24, "2"), // decimal_integer_literal
                ]),
                tree!(24, "4"), // decimal_integer_literal
                tree!(24, "4"), // decimal_integer_literal
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns <code>true</code> if the specified point (xx, yy) in Java2D\\n     * space falls within the \"hot spot\" for the specified data item, and\\n     * <code>false</code> otherwise.\\n     *\\n     * @param xx\\n     * @param yy\\n     * @param g2\\n     * @param dataArea\\n     * @param plot\\n     * @param domainAxis\\n     * @param rangeAxis\\n     * @param dataset\\n     * @param row\\n     * @param column\\n     * @param selected\\n     *\\n     * @return\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "boolean"), // type
        tree!(5, "hitTest"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "xx"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "yy"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemRendererState"), // type
            tree!(5, "state"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "bounds"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "createHotSpotBounds"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "g2"), // identifier
                  tree!(5, "dataArea"), // identifier
                  tree!(5, "plot"), // identifier
                  tree!(5, "domainAxis"), // identifier
                  tree!(5, "rangeAxis"), // identifier
                  tree!(5, "dataset"), // identifier
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                  tree!(5, "state"), // identifier
                  tree!(44, "null"), // null_literal
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "bounds"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(49, "// FIXME:  if the following test passes, we should then do the more"), // line_comment
          tree!(49, "// expensive test against the hotSpotShape"), // line_comment
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "bounds"), // identifier
              tree!(5, "contains"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "xx"), // identifier
                tree!(5, "yy"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
    ]),
  ]),
]);

    let dst_tr = tree!(1; [ // program
  tree!(2, "\\n * JFreeChart : a free chart library for the Java(tm) platform\\n * ===========================================================\\n *\\n * (C) Copyright 2000-2010, by Object Refinery Limited and Contributors.\\n *\\n * Project Info:  http://www.jfree.org/jfreechart/index.html\\n *\\n * This library is free software; you can redistribute it and/or modify it\\n * under the terms of the GNU Lesser General Public License as published by\\n * the Free Software Foundation; either version 2.1 of the License, or\\n * (at your option) any later version.\\n *\\n * This library is distributed in the hope that it will be useful, but\\n * WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY\\n * or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public\\n * License for more details.\\n *\\n * You should have received a copy of the GNU Lesser General Public\\n * License along with this library; if not, write to the Free Software\\n * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301,\\n * USA.\\n *\\n * [Java is a trademark or registered trademark of Sun Microsystems, Inc.\\n * in the United States and other countries.]\\n *\\n * ---------------------------------\\n * AbstractCategoryItemRenderer.java\\n * ---------------------------------\\n * (C) Copyright 2002-2010, by Object Refinery Limited.\\n *\\n * Original Author:  David Gilbert (for Object Refinery Limited);\\n * Contributor(s):   Richard Atkinson;\\n *                   Peter Kolb (patch 2497611);\\n *\\n * Changes:\\n * --------\\n * 29-May-2002 : Version 1 (DG);\\n * 06-Jun-2002 : Added accessor methods for the tool tip generator (DG);\\n * 11-Jun-2002 : Made constructors protected (DG);\\n * 26-Jun-2002 : Added axis to initialise method (DG);\\n * 05-Aug-2002 : Added urlGenerator member variable plus accessors (RA);\\n * 22-Aug-2002 : Added categoriesPaint attribute, based on code submitted by\\n *               Janet Banks.  This can be used when there is only one series,\\n *               and you want each category item to have a different color (DG);\\n * 01-Oct-2002 : Fixed errors reported by Checkstyle (DG);\\n * 29-Oct-2002 : Fixed bug where background image for plot was not being\\n *               drawn (DG);\\n * 05-Nov-2002 : Replaced references to CategoryDataset with TableDataset (DG);\\n * 26-Nov 2002 : Replaced the isStacked() method with getRangeType() (DG);\\n * 09-Jan-2003 : Renamed grid-line methods (DG);\\n * 17-Jan-2003 : Moved plot classes into separate package (DG);\\n * 25-Mar-2003 : Implemented Serializable (DG);\\n * 12-May-2003 : Modified to take into account the plot orientation (DG);\\n * 12-Aug-2003 : Very minor javadoc corrections (DB)\\n * 13-Aug-2003 : Implemented Cloneable (DG);\\n * 16-Sep-2003 : Changed ChartRenderingInfo --> PlotRenderingInfo (DG);\\n * 05-Nov-2003 : Fixed marker rendering bug (833623) (DG);\\n * 21-Jan-2004 : Update for renamed method in ValueAxis (DG);\\n * 11-Feb-2004 : Modified labelling for markers (DG);\\n * 12-Feb-2004 : Updated clone() method (DG);\\n * 15-Apr-2004 : Created a new CategoryToolTipGenerator interface (DG);\\n * 05-May-2004 : Fixed bug (948310) where interval markers extend outside axis\\n *               range (DG);\\n * 14-Jun-2004 : Fixed bug in drawRangeMarker() method - now uses 'paint' and\\n *               'stroke' rather than 'outlinePaint' and 'outlineStroke' (DG);\\n * 15-Jun-2004 : Interval markers can now use GradientPaint (DG);\\n * 30-Sep-2004 : Moved drawRotatedString() from RefineryUtilities\\n *               --> TextUtilities (DG);\\n * 01-Oct-2004 : Fixed bug 1029697, problem with label alignment in\\n *               drawRangeMarker() method (DG);\\n * 07-Jan-2005 : Renamed getRangeExtent() --> findRangeBounds() (DG);\\n * 21-Jan-2005 : Modified return type of calculateRangeMarkerTextAnchorPoint()\\n *               method (DG);\\n * 08-Mar-2005 : Fixed positioning of marker labels (DG);\\n * 20-Apr-2005 : Added legend label, tooltip and URL generators (DG);\\n * 01-Jun-2005 : Handle one dimension of the marker label adjustment\\n *               automatically (DG);\\n * 09-Jun-2005 : Added utility method for adding an item entity (DG);\\n * ------------- JFREECHART 1.0.x ---------------------------------------------\\n * 01-Mar-2006 : Updated getLegendItems() to check seriesVisibleInLegend\\n *               flags (DG);\\n * 20-Jul-2006 : Set dataset and series indices in LegendItem (DG);\\n * 23-Oct-2006 : Draw outlines for interval markers (DG);\\n * 24-Oct-2006 : Respect alpha setting in markers, as suggested by Sergei\\n *               Ivanov in patch 1567843 (DG);\\n * 30-Nov-2006 : Added a check for series visibility in the getLegendItem()\\n *               method (DG);\\n * 07-Dec-2006 : Fix for equals() method (DG);\\n * 22-Feb-2007 : Added createState() method (DG);\\n * 01-Mar-2007 : Fixed interval marker drawing (patch 1670686 thanks to\\n *               Sergei Ivanov) (DG);\\n * 20-Apr-2007 : Updated getLegendItem() for renderer change, and deprecated\\n *               itemLabelGenerator, toolTipGenerator and itemURLGenerator\\n *               override fields (DG);\\n * 18-May-2007 : Set dataset and seriesKey for LegendItem (DG);\\n * 20-Jun-2007 : Removed deprecated code and removed JCommon dependencies (DG);\\n * 27-Jun-2007 : Added some new methods with 'notify' argument, renamed\\n *               methods containing 'ItemURL' to just 'URL' (DG);\\n * 06-Jul-2007 : Added annotation support (DG);\\n * 17-Jun-2008 : Apply legend shape, font and paint attributes (DG);\\n * 26-Jun-2008 : Added crosshair support (DG);\\n * 25-Nov-2008 : Fixed bug in findRangeBounds() method (DG);\\n * 14-Jan-2009 : Update initialise() to store visible series indices (PK);\\n * 21-Jan-2009 : Added drawRangeLine() method (DG);\\n * 28-Jan-2009 : Updated for changes to CategoryItemRenderer interface (DG);\\n * 27-Mar-2009 : Added new findRangeBounds() method to account for hidden\\n *               series (DG);\\n * 01-Apr-2009 : Added new addEntity() method (DG);\\n * 09-Feb-2010 : Fixed bug 2947660 (DG);\\n *\\n */"), // block_comment
  tree!(3; [ // package_declaration
    tree!(4, "package"), // package
    tree!(5, "org.jfree.chart.renderer.category"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.AlphaComposite"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Composite"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Font"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.GradientPaint"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Graphics2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Paint"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Rectangle"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Shape"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.Stroke"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Ellipse2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Line2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Point2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.awt.geom.Rectangle2D"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.Serializable"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.ArrayList"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Iterator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.List"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.ChartRenderingInfo"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.LegendItem"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.LegendItemCollection"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.RenderingSource"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.annotations.CategoryAnnotation"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.axis.CategoryAxis"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.axis.ValueAxis"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.entity.CategoryItemEntity"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.entity.EntityCollection"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.event.RendererChangeEvent"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.CategoryItemLabelGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.CategorySeriesLabelGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.CategoryToolTipGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.ItemLabelPosition"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.labels.StandardCategorySeriesLabelGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.CategoryCrosshairState"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.CategoryMarker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.CategoryPlot"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.DrawingSupplier"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.IntervalMarker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.Marker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.PlotOrientation"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.PlotRenderingInfo"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.plot.ValueMarker"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.renderer.AbstractRenderer"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.text.TextUtilities"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.urls.CategoryURLGenerator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.GradientPaintTransformer"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.Layer"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.LengthAdjustmentType"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.ObjectList"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.ObjectUtilities"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.PublicCloneable"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.RectangleAnchor"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.RectangleEdge"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.RectangleInsets"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.chart.util.SortOrder"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.Range"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.category.CategoryDataset"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.category.CategoryDatasetSelectionState"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.category.SelectableCategoryDataset"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "org.jfree.data.general.DatasetUtilities"), // identifier
  ]),
  tree!(2, "\\n * An abstract base class that you can use to implement a new\\n * {@link CategoryItemRenderer}.  When you create a new\\n * {@link CategoryItemRenderer} you are not required to extend this class,\\n * but it makes the job easier.\\n */"), // block_comment
  tree!(7; [ // type_declaration
    tree!(8; [ // modifiers
      tree!(9, "public"), // visibility
      tree!(10, "abstract"), // abstract
    ]),
    tree!(11, "class"), // type_keyword
    tree!(5, "AbstractCategoryItemRenderer"), // identifier
    tree!(12; [ // superclass
      tree!(13, "extends"), // extends
      tree!(14, "AbstractRenderer"), // type
    ]),
    tree!(15; [ // super_interfaces
      tree!(16, "implements"), // implements
      tree!(17; [ // type_list
        tree!(14, "CategoryItemRenderer"), // type
        tree!(14, "Cloneable"), // type
        tree!(14, "PublicCloneable"), // type
        tree!(14, "Serializable"), // type
      ]),
    ]),
    tree!(18; [ // type_body
      tree!(2, "/** For serialization. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
          tree!(20, "static"), // static
          tree!(21, "final"), // final
        ]),
        tree!(14, "long"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "serialVersionUID"), // identifier
          tree!(23, "="), // affectation_operator
          tree!(24, "1247553218442497391L"), // decimal_integer_literal
        ]),
      ]),
      tree!(2, "/** The plot that the renderer is assigned to. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryPlot"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "plot"), // identifier
        ]),
      ]),
      tree!(2, "/** A list of item label generators (one per series). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "ObjectList"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "itemLabelGeneratorList"), // identifier
        ]),
      ]),
      tree!(2, "/** The base item label generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "baseItemLabelGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** A list of tool tip generators (one per series). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "ObjectList"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "toolTipGeneratorList"), // identifier
        ]),
      ]),
      tree!(2, "/** The base tool tip generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "baseToolTipGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** A list of label generators (one per series). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "ObjectList"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "urlGeneratorList"), // identifier
        ]),
      ]),
      tree!(2, "/** The base label generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "baseURLGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** The legend item label generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "legendItemLabelGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** The legend item tool tip generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "legendItemToolTipGenerator"), // identifier
        ]),
      ]),
      tree!(2, "/** The legend item URL generator. */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "legendItemURLGenerator"), // identifier
        ]),
      ]),
      tree!(2, "    \\n     * Annotations to be drawn in the background layer ('underneath' the data\\n     * items).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "List"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "backgroundAnnotations"), // identifier
        ]),
      ]),
      tree!(2, "    \\n     * Annotations to be drawn in the foreground layer ('on top' of the data\\n     * items).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
        ]),
        tree!(14, "List"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "foregroundAnnotations"), // identifier
        ]),
      ]),
      tree!(2, "/** The number of rows in the dataset (temporary record). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
          tree!(25, "transient"), // transient
        ]),
        tree!(14, "int"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "rowCount"), // identifier
        ]),
      ]),
      tree!(2, "/** The number of columns in the dataset (temporary record). */"), // block_comment
      tree!(19; [ // field_declaration
        tree!(8; [ // modifiers
          tree!(9, "private"), // visibility
          tree!(25, "transient"), // transient
        ]),
        tree!(14, "int"), // type
        tree!(22; [ // variable_declarator
          tree!(5, "columnCount"), // identifier
        ]),
      ]),
      tree!(2, "    \\n     * Creates a new renderer with no tool tip generator and no URL generator.\\n     * The defaults (no tool tip or URL generators) have been chosen to\\n     * minimise the processing required to generate a default chart.  If you\\n     * require tool tips or URLs, then you can easily add the required\\n     * generators.\\n     */"), // block_comment
      tree!(26; [ // constructor_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(5, "AbstractCategoryItemRenderer"), // identifier
        tree!(27), // formal_parameters
        tree!(28; [ // constructor_body
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "itemLabelGeneratorList"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ObjectList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "toolTipGeneratorList"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ObjectList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "urlGeneratorList"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ObjectList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemLabelGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "StandardCategorySeriesLabelGenerator"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "backgroundAnnotations"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ArrayList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "foregroundAnnotations"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "ArrayList"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the number of passes through the dataset required by the\\n     * renderer.  This method returns <code>1</code>, subclasses should\\n     * override if they need more passes.\\n     *\\n     * @return The pass count.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "getPassCount"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(24, "1"), // decimal_integer_literal
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the plot that the renderer has been assigned to (where\\n     * <code>null</code> indicates that the renderer is not currently assigned\\n     * to a plot).\\n     *\\n     * @return The plot (possibly <code>null</code>).\\n     *\\n     * @see #setPlot(CategoryPlot)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryPlot"), // type
        tree!(5, "getPlot"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "plot"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the plot that the renderer has been assigned to.  This method is\\n     * usually called by the {@link CategoryPlot}, in normal usage you\\n     * shouldn't need to call this method directly.\\n     *\\n     * @param plot  the plot (<code>null</code> not permitted).\\n     *\\n     * @see #getPlot()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setPlot"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "plot"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'plot' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "plot"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "plot"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// ITEM LABEL GENERATOR"), // line_comment
      tree!(2, "    \\n     * Returns the item label generator for a data item.  This implementation\\n     * returns the series item label generator if one is defined, otherwise\\n     * it returns the default item label generator (which may be\\n     * <code>null</code>).\\n     *\\n     * @param row  the row index (zero based).\\n     * @param column  the column index (zero based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(5, "getItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "CategoryItemLabelGenerator"), // type
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "itemLabelGeneratorList"), // identifier
                  ]),
                  tree!(5, "get"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "row"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "generator"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "baseItemLabelGenerator"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "generator"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the item label generator for a series.\\n     *\\n     * @param series  the series index (zero based).\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @see #setSeriesItemLabelGenerator(int, CategoryItemLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(5, "getSeriesItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(51; [ // cast_expression
              tree!(14, "CategoryItemLabelGenerator"), // type
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "itemLabelGeneratorList"), // identifier
                ]),
                tree!(5, "get"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the item label generator for a series and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getSeriesItemLabelGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setSeriesItemLabelGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the item label generator for a series and, if requested, sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getSeriesItemLabelGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "itemLabelGeneratorList"), // identifier
              ]),
              tree!(5, "set"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the base item label generator.\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @see #setBaseItemLabelGenerator(CategoryItemLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemLabelGenerator"), // type
        tree!(5, "getBaseItemLabelGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "baseItemLabelGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item label generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getBaseItemLabelGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setBaseItemLabelGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item label generator and, if requested, sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getBaseItemLabelGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "baseItemLabelGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// TOOL TIP GENERATOR"), // line_comment
      tree!(2, "    \\n     * Returns the tool tip generator that should be used for the specified\\n     * item.  You can override this method if you want to return a different\\n     * generator per item.\\n     *\\n     * @param row  the row index (zero-based).\\n     * @param column  the column index (zero-based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return The generator (possibly <code>null</code>).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(5, "getToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getSeriesToolTipGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "result"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "baseToolTipGenerator"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the tool tip generator for the specified series (a \"layer 1\"\\n     * generator).\\n     *\\n     * @param series  the series index (zero-based).\\n     *\\n     * @return The tool tip generator (possibly <code>null</code>).\\n     *\\n     * @see #setSeriesToolTipGenerator(int, CategoryToolTipGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(5, "getSeriesToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(51; [ // cast_expression
              tree!(14, "CategoryToolTipGenerator"), // type
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "toolTipGeneratorList"), // identifier
                ]),
                tree!(5, "get"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the tool tip generator for a series and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero-based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getSeriesToolTipGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setSeriesToolTipGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the tool tip generator for a series and sends a\\n     * {@link org.jfree.chart.event.RendererChangeEvent} to all registered\\n     * listeners.\\n     *\\n     * @param series  the series index (zero-based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getSeriesToolTipGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "toolTipGeneratorList"), // identifier
              ]),
              tree!(5, "set"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the base tool tip generator (the \"layer 2\" generator).\\n     *\\n     * @return The tool tip generator (possibly <code>null</code>).\\n     *\\n     * @see #setBaseToolTipGenerator(CategoryToolTipGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryToolTipGenerator"), // type
        tree!(5, "getBaseToolTipGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "baseToolTipGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base tool tip generator and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getBaseToolTipGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setBaseToolTipGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base tool tip generator and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getBaseToolTipGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "baseToolTipGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// URL GENERATOR"), // line_comment
      tree!(2, "    \\n     * Returns the URL generator for a data item.\\n     *\\n     * @param row  the row index (zero based).\\n     * @param column  the column index (zero based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return The URL generator.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(5, "getURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryURLGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "CategoryURLGenerator"), // type
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "urlGeneratorList"), // identifier
                  ]),
                  tree!(5, "get"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "row"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "generator"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "baseURLGenerator"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "generator"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the URL generator for a series.\\n     *\\n     * @param series  the series index (zero based).\\n     *\\n     * @return The URL generator for the series.\\n     *\\n     * @see #setSeriesURLGenerator(int, CategoryURLGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(5, "getSeriesURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(51; [ // cast_expression
              tree!(14, "CategoryURLGenerator"), // type
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "urlGeneratorList"), // identifier
                ]),
                tree!(5, "get"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the URL generator for a series and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator.\\n     *\\n     * @see #getSeriesURLGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setSeriesURLGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the URL generator for a series and, if requested, sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param series  the series index (zero based).\\n     * @param generator  the generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @since 1.2.0\\n     *\\n     * @see #getSeriesURLGenerator(int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setSeriesURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "urlGeneratorList"), // identifier
              ]),
              tree!(5, "set"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
                tree!(5, "generator"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the base item URL generator.\\n     *\\n     * @return The item URL generator.\\n     *\\n     * @see #setBaseURLGenerator(CategoryURLGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryURLGenerator"), // type
        tree!(5, "getBaseURLGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "baseURLGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item URL generator.\\n     *\\n     * @param generator  the item URL generator.\\n     *\\n     * @see #getBaseURLGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setBaseURLGenerator"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "generator"), // identifier
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the base item URL generator.\\n     *\\n     * @param generator  the item URL generator (<code>null</code> permitted).\\n     * @param notify  notify listeners?\\n     *\\n     * @see #getBaseURLGenerator()\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setBaseURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryURLGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "notify"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "baseURLGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(5, "notify"), // identifier
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(49, "// ANNOTATIONS"), // line_comment
      tree!(2, "    \\n     * Adds an annotation and sends a {@link RendererChangeEvent} to all\\n     * registered listeners.  The annotation is added to the foreground\\n     * layer.\\n     *\\n     * @param annotation  the annotation (<code>null</code> not permitted).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addAnnotation"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAnnotation"), // type
            tree!(5, "annotation"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(49, "// defer argument checking"), // line_comment
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "addAnnotation"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "annotation"), // identifier
                tree!(31; [ // field_access
                  tree!(5, "Layer"), // identifier
                  tree!(5, "FOREGROUND"), // identifier
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Adds an annotation to the specified layer.\\n     *\\n     * @param annotation  the annotation (<code>null</code> not permitted).\\n     * @param layer  the layer (<code>null</code> not permitted).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addAnnotation"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAnnotation"), // type
            tree!(5, "annotation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Layer"), // type
            tree!(5, "layer"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "annotation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'annotation' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "layer"), // identifier
                tree!(5, "equals"), // identifier
                tree!(35; [ // argument_list
                  tree!(31; [ // field_access
                    tree!(5, "Layer"), // identifier
                    tree!(5, "FOREGROUND"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "foregroundAnnotations"), // identifier
                  ]),
                  tree!(5, "add"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "annotation"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "notifyListeners"), // identifier
                  tree!(35; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "RendererChangeEvent"), // type
                      tree!(35; [ // argument_list
                        tree!(32, "this"), // this
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(52; [ // method_invocation
                  tree!(5, "layer"), // identifier
                  tree!(5, "equals"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(5, "Layer"), // identifier
                      tree!(5, "BACKGROUND"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "backgroundAnnotations"), // identifier
                    ]),
                    tree!(5, "add"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "annotation"), // identifier
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "notifyListeners"), // identifier
                    tree!(35; [ // argument_list
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "RendererChangeEvent"), // type
                        tree!(35; [ // argument_list
                          tree!(32, "this"), // this
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(49, "// should never get here"), // line_comment
                tree!(45; [ // throw_statement
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "RuntimeException"), // type
                    tree!(35; [ // argument_list
                      tree!(46; [ // string_literal
                        tree!(47, "\""), // "
                        tree!(48, "Unknown layer."), // string_fragment
                        tree!(47, "\""), // "
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Removes the specified annotation and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @param annotation  the annotation to remove (<code>null</code> not\\n     *                    permitted).\\n     *\\n     * @return A boolean to indicate whether or not the annotation was\\n     *         successfully removed.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "boolean"), // type
        tree!(5, "removeAnnotation"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAnnotation"), // type
            tree!(5, "annotation"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "boolean"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "removed"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "foregroundAnnotations"), // identifier
                ]),
                tree!(5, "remove"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "annotation"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(5, "removed"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(42; [ // binary_expression
                tree!(5, "removed"), // identifier
                tree!(54, "&"), // bitwise_operator
                tree!(52; [ // method_invocation
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "backgroundAnnotations"), // identifier
                  ]),
                  tree!(5, "remove"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "annotation"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "notifyListeners"), // identifier
              tree!(35; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "RendererChangeEvent"), // type
                  tree!(35; [ // argument_list
                    tree!(32, "this"), // this
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "removed"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Removes all annotations and sends a {@link RendererChangeEvent}\\n     * to all registered listeners.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "removeAnnotations"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "foregroundAnnotations"), // identifier
              ]),
              tree!(5, "clear"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "backgroundAnnotations"), // identifier
              ]),
              tree!(5, "clear"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "notifyListeners"), // identifier
              tree!(35; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "RendererChangeEvent"), // type
                  tree!(35; [ // argument_list
                    tree!(32, "this"), // this
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the legend item label generator.\\n     *\\n     * @return The label generator (never <code>null</code>).\\n     *\\n     * @see #setLegendItemLabelGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(5, "getLegendItemLabelGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "legendItemLabelGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the legend item label generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> not permitted).\\n     *\\n     * @see #getLegendItemLabelGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setLegendItemLabelGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategorySeriesLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'generator' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemLabelGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "fireChangeEvent"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the legend item tool tip generator.\\n     *\\n     * @return The tool tip generator (possibly <code>null</code>).\\n     *\\n     * @see #setLegendItemToolTipGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(5, "getLegendItemToolTipGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "legendItemToolTipGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the legend item tool tip generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #setLegendItemToolTipGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setLegendItemToolTipGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategorySeriesLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemToolTipGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "fireChangeEvent"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the legend item URL generator.\\n     *\\n     * @return The URL generator (possibly <code>null</code>).\\n     *\\n     * @see #setLegendItemURLGenerator(CategorySeriesLabelGenerator)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategorySeriesLabelGenerator"), // type
        tree!(5, "getLegendItemURLGenerator"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "legendItemURLGenerator"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Sets the legend item URL generator and sends a\\n     * {@link RendererChangeEvent} to all registered listeners.\\n     *\\n     * @param generator  the generator (<code>null</code> permitted).\\n     *\\n     * @see #getLegendItemURLGenerator()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "setLegendItemURLGenerator"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategorySeriesLabelGenerator"), // type
            tree!(5, "generator"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(30; [ // assignment_expression
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "legendItemURLGenerator"), // identifier
              ]),
              tree!(23, "="), // affectation_operator
              tree!(5, "generator"), // identifier
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "fireChangeEvent"), // identifier
              tree!(35), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the number of rows in the dataset.  This value is updated in the\\n     * {@link AbstractCategoryItemRenderer#initialise} method.\\n     *\\n     * @return The row count.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "getRowCount"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "rowCount"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the number of columns in the dataset.  This value is updated in\\n     * the {@link AbstractCategoryItemRenderer#initialise} method.\\n     *\\n     * @return The column count.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "getColumnCount"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(31; [ // field_access
              tree!(32, "this"), // this
              tree!(5, "columnCount"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Creates a new state instance---this method is called from the\\n     * {@link #initialise(Graphics2D, Rectangle2D, CategoryPlot, int,\\n     * PlotRenderingInfo)} method.  Subclasses can override this method if\\n     * they need to use a subclass of {@link CategoryItemRendererState}.\\n     *\\n     * @param info  collects plot rendering info (<code>null</code> permitted).\\n     *\\n     * @return The new state instance (never <code>null</code>).\\n     *\\n     * @since 1.0.5\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "CategoryItemRendererState"), // type
        tree!(5, "createState"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "PlotRenderingInfo"), // type
            tree!(5, "info"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemRendererState"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "state"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "CategoryItemRendererState"), // type
                tree!(35; [ // argument_list
                  tree!(5, "info"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int[]"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "visibleSeriesTemp"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(55; [ // array_creation_expression
                tree!(34, "new"), // new
                tree!(14, "int"), // type
                tree!(56; [ // dimensions_expr
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "rowCount"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "visibleSeriesCount"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(24, "0"), // decimal_integer_literal
            ]),
          ]),
          tree!(57; [ // for_statement
            tree!(50; [ // local_variable_declaration
              tree!(14, "int"), // type
              tree!(22; [ // variable_declarator
                tree!(5, "row"), // identifier
                tree!(23, "="), // affectation_operator
                tree!(24, "0"), // decimal_integer_literal
              ]),
            ]),
            tree!(42; [ // binary_expression
              tree!(5, "row"), // identifier
              tree!(43, "<"), // comparison_operator
              tree!(31; [ // field_access
                tree!(32, "this"), // this
                tree!(5, "rowCount"), // identifier
              ]),
            ]),
            tree!(58; [ // update_expression
              tree!(5, "row"), // identifier
              tree!(59, "++"), // increment_operator
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(52; [ // method_invocation
                    tree!(5, "isSeriesVisible"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(60; [ // array_access
                        tree!(5, "visibleSeriesTemp"), // identifier
                        tree!(5, "visibleSeriesCount"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(5, "row"), // identifier
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(58; [ // update_expression
                      tree!(5, "visibleSeriesCount"), // identifier
                      tree!(59, "++"), // increment_operator
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int[]"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "visibleSeries"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(55; [ // array_creation_expression
                tree!(34, "new"), // new
                tree!(14, "int"), // type
                tree!(56; [ // dimensions_expr
                  tree!(5, "visibleSeriesCount"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "System"), // identifier
              tree!(5, "arraycopy"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "visibleSeriesTemp"), // identifier
                tree!(24, "0"), // decimal_integer_literal
                tree!(5, "visibleSeries"), // identifier
                tree!(24, "0"), // decimal_integer_literal
                tree!(5, "visibleSeriesCount"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "state"), // identifier
              tree!(5, "setVisibleSeriesArray"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "visibleSeries"), // identifier
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "state"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Initialises the renderer and returns a state object that will be used\\n     * for the remainder of the drawing process for a single chart.  The state\\n     * object allows for the fact that the renderer may be used simultaneously\\n     * by multiple threads (each thread will work with a separate state object).\\n     *\\n     * @param g2  the graphics device.\\n     * @param dataArea  the data area.\\n     * @param plot  the plot.\\n     * @param info  an object for returning information about the structure of\\n     *              the plot (<code>null</code> permitted).\\n     *\\n     * @return The renderer state.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "CategoryItemRendererState"), // type
        tree!(5, "initialise"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotRenderingInfo"), // type
            tree!(5, "info"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "setPlot"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "plot"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "dataset"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "rowCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getRowCount"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "columnCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getColumnCount"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "rowCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(24, "0"), // decimal_integer_literal
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(32, "this"), // this
                    tree!(5, "columnCount"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(24, "0"), // decimal_integer_literal
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemRendererState"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "state"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "createState"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "info"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(49, "// determine if there is any selection state for the dataset"), // line_comment
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDatasetSelectionState"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "selectionState"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(5, "dataset"), // identifier
                tree!(62, "instanceof"), // instanceof
                tree!(14, "SelectableCategoryDataset"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "SelectableCategoryDataset"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "scd"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "SelectableCategoryDataset"), // type
                    tree!(5, "dataset"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "selectionState"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "scd"), // identifier
                    tree!(5, "getSelectionState"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(49, "// if the selection state is still null, go to the selection source"), // line_comment
          tree!(49, "// and ask if it has state..."), // line_comment
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(42; [ // binary_expression
                  tree!(5, "selectionState"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(44, "null"), // null_literal
                ]),
                tree!(63, "&&"), // logical_operator
                tree!(42; [ // binary_expression
                  tree!(5, "info"), // identifier
                  tree!(43, "!="), // comparison_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "ChartRenderingInfo"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "cri"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "info"), // identifier
                    tree!(5, "getOwner"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "cri"), // identifier
                    tree!(43, "!="), // comparison_operator
                    tree!(44, "null"), // null_literal
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "RenderingSource"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "rs"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "cri"), // identifier
                        tree!(5, "getRenderingSource"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "selectionState"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryDatasetSelectionState"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "rs"), // identifier
                          tree!(5, "getSelectionState"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "dataset"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "state"), // identifier
              tree!(5, "setSelectionState"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "selectionState"), // identifier
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "state"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the range of values the renderer requires to display all the\\n     * items from the specified dataset.\\n     *\\n     * @param dataset  the dataset (<code>null</code> permitted).\\n     *\\n     * @return The range (or <code>null</code> if the dataset is\\n     *         <code>null</code> or empty).\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Range"), // type
        tree!(5, "findRangeBounds"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "findRangeBounds"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "dataset"), // identifier
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the range of values the renderer requires to display all the\\n     * items from the specified dataset.\\n     *\\n     * @param dataset  the dataset (<code>null</code> permitted).\\n     * @param includeInterval  include the y-interval if the dataset has one.\\n     *\\n     * @return The range (<code>null</code> if the dataset is <code>null</code>\\n     *         or empty).\\n     *\\n     * @since 1.0.13\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "Range"), // type
        tree!(5, "findRangeBounds"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "includeInterval"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "dataset"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "getDataBoundsIncludesVisibleSeriesOnly"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "List"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "visibleSeriesKeys"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "ArrayList"), // type
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "int"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "seriesCount"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getRowCount"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(57; [ // for_statement
                tree!(50; [ // local_variable_declaration
                  tree!(14, "int"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "s"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(24, "0"), // decimal_integer_literal
                  ]),
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "s"), // identifier
                  tree!(43, "<"), // comparison_operator
                  tree!(5, "seriesCount"), // identifier
                ]),
                tree!(58; [ // update_expression
                  tree!(5, "s"), // identifier
                  tree!(59, "++"), // increment_operator
                ]),
                tree!(37; [ // block
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(52; [ // method_invocation
                        tree!(5, "isSeriesVisible"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "s"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(29; [ // expression_statement
                        tree!(52; [ // method_invocation
                          tree!(5, "visibleSeriesKeys"), // identifier
                          tree!(5, "add"), // identifier
                          tree!(35; [ // argument_list
                            tree!(52; [ // method_invocation
                              tree!(5, "dataset"), // identifier
                              tree!(5, "getRowKey"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "s"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(38; [ // return_statement
                tree!(52; [ // method_invocation
                  tree!(5, "DatasetUtilities"), // identifier
                  tree!(5, "findRangeBounds"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "dataset"), // identifier
                    tree!(5, "visibleSeriesKeys"), // identifier
                    tree!(5, "includeInterval"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(52; [ // method_invocation
                  tree!(5, "DatasetUtilities"), // identifier
                  tree!(5, "findRangeBounds"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "dataset"), // identifier
                    tree!(5, "includeInterval"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the Java2D coordinate for the middle of the specified data item.\\n     *\\n     * @param rowKey  the row key.\\n     * @param columnKey  the column key.\\n     * @param dataset  the dataset.\\n     * @param axis  the axis.\\n     * @param area  the data area.\\n     * @param edge  the edge along which the axis lies.\\n     *\\n     * @return The Java2D coordinate for the middle of the item.\\n     *\\n     * @since 1.0.11\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(65; [ // floating_point_type
          tree!(66, "double"), // double
        ]),
        tree!(5, "getItemMiddle"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "rowKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "columnKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "area"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleEdge"), // type
            tree!(5, "edge"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "axis"), // identifier
              tree!(5, "getCategoryMiddle"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "columnKey"), // identifier
                tree!(52; [ // method_invocation
                  tree!(5, "dataset"), // identifier
                  tree!(5, "getColumnKeys"), // identifier
                  tree!(35), // argument_list
                ]),
                tree!(5, "area"), // identifier
                tree!(5, "edge"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a background for the data area.  The default implementation just\\n     * gets the plot to draw the background, but some renderers will override\\n     * this behaviour.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param dataArea  the data area.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawBackground"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "plot"), // identifier
              tree!(5, "drawBackground"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "g2"), // identifier
                tree!(5, "dataArea"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws an outline for the data area.  The default implementation just\\n     * gets the plot to draw the outline, but some renderers will override this\\n     * behaviour.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param dataArea  the data area.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawOutline"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "plot"), // identifier
              tree!(5, "drawOutline"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "g2"), // identifier
                tree!(5, "dataArea"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a grid line against the domain axis.\\n     * <P>\\n     * Note that this default implementation assumes that the horizontal axis\\n     * is the domain axis. If this is not the case, you will need to override\\n     * this method.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param dataArea  the area for plotting data (not yet adjusted for any\\n     *                  3D effect).\\n     * @param value  the Java2D value at which the grid line should be drawn.\\n     * @param paint  the paint (<code>null</code> not permitted).\\n     * @param stroke  the stroke (<code>null</code> not permitted).\\n     *\\n     * @see #drawRangeGridline(Graphics2D, CategoryPlot, ValueAxis,\\n     *     Rectangle2D, double)\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawDomainLine"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "value"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Paint"), // type
            tree!(5, "paint"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Stroke"), // type
            tree!(5, "stroke"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "paint"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'paint' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "stroke"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'stroke' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Line2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "line"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "PlotOrientation"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "orientation"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getOrientation"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "Line2D.Double"), // type
                    tree!(35; [ // argument_list
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMinX"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "value"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMaxX"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "value"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "line"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "Line2D.Double"), // type
                      tree!(35; [ // argument_list
                        tree!(5, "value"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMinY"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(5, "value"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMaxY"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setPaint"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "paint"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setStroke"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "stroke"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "draw"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "line"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a line perpendicular to the range axis.\\n     *\\n     * @param g2  the graphics device.\\n     * @param plot  the plot.\\n     * @param axis  the value axis.\\n     * @param dataArea  the area for plotting data (not yet adjusted for any 3D\\n     *                  effect).\\n     * @param value  the value at which the grid line should be drawn.\\n     * @param paint  the paint (<code>null</code> not permitted).\\n     * @param stroke  the stroke (<code>null</code> not permitted).\\n     *\\n     * @see #drawRangeGridline\\n     *\\n     * @since 1.0.13\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawRangeLine"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "value"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Paint"), // type
            tree!(5, "paint"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Stroke"), // type
            tree!(5, "stroke"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Range"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "range"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "axis"), // identifier
                tree!(5, "getRange"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "range"), // identifier
                  tree!(5, "contains"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "value"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38), // return_statement
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "PlotOrientation"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "orientation"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getOrientation"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Line2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "line"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(22; [ // variable_declarator
              tree!(5, "v"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "axis"), // identifier
                tree!(5, "valueToJava2D"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "value"), // identifier
                  tree!(5, "dataArea"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getRangeAxisEdge"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "Line2D.Double"), // type
                    tree!(35; [ // argument_list
                      tree!(5, "v"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMinY"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "v"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataArea"), // identifier
                        tree!(5, "getMaxY"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "line"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "Line2D.Double"), // type
                      tree!(35; [ // argument_list
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMinX"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(5, "v"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "dataArea"), // identifier
                          tree!(5, "getMaxX"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(5, "v"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setPaint"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "paint"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setStroke"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "stroke"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "draw"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "line"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a marker for the domain axis.\\n     *\\n     * @param g2  the graphics device (not <code>null</code>).\\n     * @param plot  the plot (not <code>null</code>).\\n     * @param axis  the range axis (not <code>null</code>).\\n     * @param marker  the marker to be drawn (not <code>null</code>).\\n     * @param dataArea  the area inside the axes (not <code>null</code>).\\n     *\\n     * @see #drawRangeMarker(Graphics2D, CategoryPlot, ValueAxis, Marker,\\n     *     Rectangle2D)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawDomainMarker"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryMarker"), // type
            tree!(5, "marker"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Comparable"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "category"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getKey"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDataset"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "dataset"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getDataset"), // identifier
                tree!(35; [ // argument_list
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getIndexOf"), // identifier
                    tree!(35; [ // argument_list
                      tree!(32, "this"), // this
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "columnIndex"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getColumnIndex"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "category"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "columnIndex"), // identifier
                tree!(43, "<"), // comparison_operator
                tree!(24, "0"), // decimal_integer_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38), // return_statement
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(8; [ // modifiers
              tree!(21, "final"), // final
            ]),
            tree!(14, "Composite"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "savedComposite"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "g2"), // identifier
                tree!(5, "getComposite"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setComposite"), // identifier
              tree!(35; [ // argument_list
                tree!(52; [ // method_invocation
                  tree!(5, "AlphaComposite"), // identifier
                  tree!(5, "getInstance"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(5, "AlphaComposite"), // identifier
                      tree!(5, "SRC_OVER"), // identifier
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getAlpha"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "PlotOrientation"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "orientation"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getOrientation"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "bounds"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getDrawAsLine"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getCategoryMiddle"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "columnIndex"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataset"), // identifier
                        tree!(5, "getColumnCount"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getDomainAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Line2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "orientation"), // identifier
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "HORIZONTAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "line"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Line2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMinX"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMaxX"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "VERTICAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "line"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Line2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(5, "v"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "v"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setStroke"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getStroke"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "draw"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "line"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "bounds"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "line"), // identifier
                    tree!(5, "getBounds2D"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v0"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getCategoryStart"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "columnIndex"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataset"), // identifier
                        tree!(5, "getColumnCount"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getDomainAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v1"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getCategoryEnd"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "columnIndex"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "dataset"), // identifier
                        tree!(5, "getColumnCount"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getDomainAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Rectangle2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "area"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "orientation"), // identifier
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "HORIZONTAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "area"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Rectangle2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMinX"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v0"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getWidth"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(41; [ // parenthesized_expression
                            tree!(42; [ // binary_expression
                              tree!(5, "v1"), // identifier
                              tree!(69, "-"), // arithmetic_operator
                              tree!(5, "v0"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "VERTICAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "area"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Rectangle2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(5, "v0"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(41; [ // parenthesized_expression
                              tree!(42; [ // binary_expression
                                tree!(5, "v1"), // identifier
                                tree!(69, "-"), // arithmetic_operator
                                tree!(5, "v0"), // identifier
                              ]),
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getHeight"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "fill"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "area"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "bounds"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(5, "area"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "label"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getLabel"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "RectangleAnchor"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "anchor"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "marker"), // identifier
                tree!(5, "getLabelAnchor"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "label"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "Font"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "labelFont"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "marker"), // identifier
                    tree!(5, "getLabelFont"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setFont"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "labelFont"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabelPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Point2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "coordinates"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "calculateDomainMarkerTextAnchorPoint"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "g2"), // identifier
                      tree!(5, "orientation"), // identifier
                      tree!(5, "dataArea"), // identifier
                      tree!(5, "bounds"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "marker"), // identifier
                        tree!(5, "getLabelOffset"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "marker"), // identifier
                        tree!(5, "getLabelOffsetType"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "anchor"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "TextUtilities"), // identifier
                  tree!(5, "drawAlignedString"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "label"), // identifier
                    tree!(5, "g2"), // identifier
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "coordinates"), // identifier
                        tree!(5, "getX"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "coordinates"), // identifier
                        tree!(5, "getY"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabelTextAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "g2"), // identifier
              tree!(5, "setComposite"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "savedComposite"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws a marker for the range axis.\\n     *\\n     * @param g2  the graphics device (not <code>null</code>).\\n     * @param plot  the plot (not <code>null</code>).\\n     * @param axis  the range axis (not <code>null</code>).\\n     * @param marker  the marker to be drawn (not <code>null</code>).\\n     * @param dataArea  the area inside the axes (not <code>null</code>).\\n     *\\n     * @see #drawDomainMarker(Graphics2D, CategoryPlot, CategoryAxis,\\n     *     CategoryMarker, Rectangle2D)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawRangeMarker"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "axis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Marker"), // type
            tree!(5, "marker"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(5, "marker"), // identifier
                tree!(62, "instanceof"), // instanceof
                tree!(14, "ValueMarker"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "ValueMarker"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "vm"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ValueMarker"), // type
                    tree!(5, "marker"), // identifier
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "value"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "vm"), // identifier
                    tree!(5, "getValue"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Range"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "range"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "getRange"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(67; [ // unary_expression
                    tree!(68, "!"), // !
                    tree!(52; [ // method_invocation
                      tree!(5, "range"), // identifier
                      tree!(5, "contains"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "value"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(38), // return_statement
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(8; [ // modifiers
                  tree!(21, "final"), // final
                ]),
                tree!(14, "Composite"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "savedComposite"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "getComposite"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setComposite"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "AlphaComposite"), // identifier
                      tree!(5, "getInstance"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(5, "AlphaComposite"), // identifier
                          tree!(5, "SRC_OVER"), // identifier
                        ]),
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getAlpha"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "PlotOrientation"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "orientation"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getOrientation"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "v"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "axis"), // identifier
                    tree!(5, "valueToJava2D"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "value"), // identifier
                      tree!(5, "dataArea"), // identifier
                      tree!(52; [ // method_invocation
                        tree!(5, "plot"), // identifier
                        tree!(5, "getRangeAxisEdge"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Line2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "line"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "orientation"), // identifier
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "HORIZONTAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "line"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Line2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(5, "v"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMinY"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(5, "v"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "getMaxY"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "VERTICAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "line"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Line2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinX"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "v"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxX"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "v"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setStroke"), // identifier
                  tree!(35; [ // argument_list
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getStroke"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "draw"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "line"), // identifier
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "String"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "label"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "marker"), // identifier
                    tree!(5, "getLabel"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "RectangleAnchor"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "anchor"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "marker"), // identifier
                    tree!(5, "getLabelAnchor"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(5, "label"), // identifier
                    tree!(43, "!="), // comparison_operator
                    tree!(44, "null"), // null_literal
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "Font"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "labelFont"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "marker"), // identifier
                        tree!(5, "getLabelFont"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "g2"), // identifier
                      tree!(5, "setFont"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "labelFont"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "g2"), // identifier
                      tree!(5, "setPaint"), // identifier
                      tree!(35; [ // argument_list
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getLabelPaint"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "Point2D"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "coordinates"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "calculateRangeMarkerTextAnchorPoint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "g2"), // identifier
                          tree!(5, "orientation"), // identifier
                          tree!(5, "dataArea"), // identifier
                          tree!(52; [ // method_invocation
                            tree!(5, "line"), // identifier
                            tree!(5, "getBounds2D"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getLabelOffset"), // identifier
                            tree!(35), // argument_list
                          ]),
                          tree!(31; [ // field_access
                            tree!(5, "LengthAdjustmentType"), // identifier
                            tree!(5, "EXPAND"), // identifier
                          ]),
                          tree!(5, "anchor"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "TextUtilities"), // identifier
                      tree!(5, "drawAlignedString"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "label"), // identifier
                        tree!(5, "g2"), // identifier
                        tree!(51; [ // cast_expression
                          tree!(65; [ // floating_point_type
                            tree!(70, "float"), // float
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "coordinates"), // identifier
                            tree!(5, "getX"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                        tree!(51; [ // cast_expression
                          tree!(65; [ // floating_point_type
                            tree!(70, "float"), // float
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "coordinates"), // identifier
                            tree!(5, "getY"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getLabelTextAnchor"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setComposite"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "savedComposite"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(61; [ // instanceof_expression
                  tree!(5, "marker"), // identifier
                  tree!(62, "instanceof"), // instanceof
                  tree!(14, "IntervalMarker"), // type
                ]),
              ]),
              tree!(37; [ // block
                tree!(50; [ // local_variable_declaration
                  tree!(14, "IntervalMarker"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "im"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(51; [ // cast_expression
                      tree!(14, "IntervalMarker"), // type
                      tree!(5, "marker"), // identifier
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "start"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "im"), // identifier
                      tree!(5, "getStartValue"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "end"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "im"), // identifier
                      tree!(5, "getEndValue"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "Range"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "range"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "axis"), // identifier
                      tree!(5, "getRange"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(67; [ // unary_expression
                      tree!(68, "!"), // !
                      tree!(41; [ // parenthesized_expression
                        tree!(52; [ // method_invocation
                          tree!(5, "range"), // identifier
                          tree!(5, "intersects"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "start"), // identifier
                            tree!(5, "end"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(38), // return_statement
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(8; [ // modifiers
                    tree!(21, "final"), // final
                  ]),
                  tree!(14, "Composite"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "savedComposite"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "g2"), // identifier
                      tree!(5, "getComposite"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "setComposite"), // identifier
                    tree!(35; [ // argument_list
                      tree!(52; [ // method_invocation
                        tree!(5, "AlphaComposite"), // identifier
                        tree!(5, "getInstance"), // identifier
                        tree!(35; [ // argument_list
                          tree!(31; [ // field_access
                            tree!(5, "AlphaComposite"), // identifier
                            tree!(5, "SRC_OVER"), // identifier
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getAlpha"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "start2d"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "axis"), // identifier
                      tree!(5, "valueToJava2D"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "start"), // identifier
                        tree!(5, "dataArea"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "plot"), // identifier
                          tree!(5, "getRangeAxisEdge"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "end2d"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "axis"), // identifier
                      tree!(5, "valueToJava2D"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "end"), // identifier
                        tree!(5, "dataArea"), // identifier
                        tree!(52; [ // method_invocation
                          tree!(5, "plot"), // identifier
                          tree!(5, "getRangeAxisEdge"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "low"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "Math"), // identifier
                      tree!(5, "min"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "start2d"), // identifier
                        tree!(5, "end2d"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(65; [ // floating_point_type
                    tree!(66, "double"), // double
                  ]),
                  tree!(22; [ // variable_declarator
                    tree!(5, "high"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "Math"), // identifier
                      tree!(5, "max"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "start2d"), // identifier
                        tree!(5, "end2d"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "PlotOrientation"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "orientation"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "plot"), // identifier
                      tree!(5, "getOrientation"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "Rectangle2D"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "rect"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(44, "null"), // null_literal
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "orientation"), // identifier
                      tree!(43, "=="), // comparison_operator
                      tree!(31; [ // field_access
                        tree!(5, "PlotOrientation"), // identifier
                        tree!(5, "HORIZONTAL"), // identifier
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(49, "// clip left and right bounds to data area"), // line_comment
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "low"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "Math"), // identifier
                          tree!(5, "max"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "low"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "high"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "Math"), // identifier
                          tree!(5, "min"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "high"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(30; [ // assignment_expression
                        tree!(5, "rect"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(33; [ // object_creation_expression
                          tree!(34, "new"), // new
                          tree!(14, "Rectangle2D.Double"), // type
                          tree!(35; [ // argument_list
                            tree!(5, "low"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(42; [ // binary_expression
                              tree!(5, "high"), // identifier
                              tree!(69, "-"), // arithmetic_operator
                              tree!(5, "low"), // identifier
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getHeight"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(42; [ // binary_expression
                        tree!(5, "orientation"), // identifier
                        tree!(43, "=="), // comparison_operator
                        tree!(31; [ // field_access
                          tree!(5, "PlotOrientation"), // identifier
                          tree!(5, "VERTICAL"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(49, "// clip top and bottom bounds to data area"), // line_comment
                      tree!(29; [ // expression_statement
                        tree!(30; [ // assignment_expression
                          tree!(5, "low"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "Math"), // identifier
                            tree!(5, "max"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "low"), // identifier
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getMinY"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(29; [ // expression_statement
                        tree!(30; [ // assignment_expression
                          tree!(5, "high"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "Math"), // identifier
                            tree!(5, "min"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "high"), // identifier
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getMaxY"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(29; [ // expression_statement
                        tree!(30; [ // assignment_expression
                          tree!(5, "rect"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(33; [ // object_creation_expression
                            tree!(34, "new"), // new
                            tree!(14, "Rectangle2D.Double"), // type
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getMinX"), // identifier
                                tree!(35), // argument_list
                              ]),
                              tree!(5, "low"), // identifier
                              tree!(52; [ // method_invocation
                                tree!(5, "dataArea"), // identifier
                                tree!(5, "getWidth"), // identifier
                                tree!(35), // argument_list
                              ]),
                              tree!(42; [ // binary_expression
                                tree!(5, "high"), // identifier
                                tree!(69, "-"), // arithmetic_operator
                                tree!(5, "low"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "Paint"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "p"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getPaint"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(61; [ // instanceof_expression
                      tree!(5, "p"), // identifier
                      tree!(62, "instanceof"), // instanceof
                      tree!(14, "GradientPaint"), // type
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "GradientPaint"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "gp"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(51; [ // cast_expression
                          tree!(14, "GradientPaint"), // type
                          tree!(5, "p"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "GradientPaintTransformer"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "t"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "im"), // identifier
                          tree!(5, "getGradientPaintTransformer"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                    tree!(40; [ // if_statement
                      tree!(41; [ // parenthesized_expression
                        tree!(42; [ // binary_expression
                          tree!(5, "t"), // identifier
                          tree!(43, "!="), // comparison_operator
                          tree!(44, "null"), // null_literal
                        ]),
                      ]),
                      tree!(37; [ // block
                        tree!(29; [ // expression_statement
                          tree!(30; [ // assignment_expression
                            tree!(5, "gp"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "t"), // identifier
                              tree!(5, "transform"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "gp"), // identifier
                                tree!(5, "rect"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setPaint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "gp"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setPaint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "p"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "fill"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "rect"), // identifier
                    ]),
                  ]),
                ]),
                tree!(49, "// now draw the outlines, if visible..."), // line_comment
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(42; [ // binary_expression
                        tree!(52; [ // method_invocation
                          tree!(5, "im"), // identifier
                          tree!(5, "getOutlinePaint"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(43, "!="), // comparison_operator
                        tree!(44, "null"), // null_literal
                      ]),
                      tree!(63, "&&"), // logical_operator
                      tree!(42; [ // binary_expression
                        tree!(52; [ // method_invocation
                          tree!(5, "im"), // identifier
                          tree!(5, "getOutlineStroke"), // identifier
                          tree!(35), // argument_list
                        ]),
                        tree!(43, "!="), // comparison_operator
                        tree!(44, "null"), // null_literal
                      ]),
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(40; [ // if_statement
                      tree!(41; [ // parenthesized_expression
                        tree!(42; [ // binary_expression
                          tree!(5, "orientation"), // identifier
                          tree!(43, "=="), // comparison_operator
                          tree!(31; [ // field_access
                            tree!(5, "PlotOrientation"), // identifier
                            tree!(5, "VERTICAL"), // identifier
                          ]),
                        ]),
                      ]),
                      tree!(37; [ // block
                        tree!(50; [ // local_variable_declaration
                          tree!(14, "Line2D"), // type
                          tree!(22; [ // variable_declarator
                            tree!(5, "line"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(14, "Line2D.Double"), // type
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "x0"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "x1"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setPaint"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlinePaint"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setStroke"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlineStroke"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "start"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "x0"), // identifier
                                  tree!(5, "start2d"), // identifier
                                  tree!(5, "x1"), // identifier
                                  tree!(5, "start2d"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "end"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "x0"), // identifier
                                  tree!(5, "end2d"), // identifier
                                  tree!(5, "x1"), // identifier
                                  tree!(5, "end2d"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(37; [ // block
                        tree!(49, "// PlotOrientation.HORIZONTAL"), // line_comment
                        tree!(50; [ // local_variable_declaration
                          tree!(14, "Line2D"), // type
                          tree!(22; [ // variable_declarator
                            tree!(5, "line"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(14, "Line2D.Double"), // type
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "y0"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMinY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(50; [ // local_variable_declaration
                          tree!(65; [ // floating_point_type
                            tree!(66, "double"), // double
                          ]),
                          tree!(22; [ // variable_declarator
                            tree!(5, "y1"), // identifier
                            tree!(23, "="), // affectation_operator
                            tree!(52; [ // method_invocation
                              tree!(5, "dataArea"), // identifier
                              tree!(5, "getMaxY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setPaint"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlinePaint"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(29; [ // expression_statement
                          tree!(52; [ // method_invocation
                            tree!(5, "g2"), // identifier
                            tree!(5, "setStroke"), // identifier
                            tree!(35; [ // argument_list
                              tree!(52; [ // method_invocation
                                tree!(5, "im"), // identifier
                                tree!(5, "getOutlineStroke"), // identifier
                                tree!(35), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "start"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "start2d"), // identifier
                                  tree!(5, "y0"), // identifier
                                  tree!(5, "start2d"), // identifier
                                  tree!(5, "y1"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(40; [ // if_statement
                          tree!(41; [ // parenthesized_expression
                            tree!(52; [ // method_invocation
                              tree!(5, "range"), // identifier
                              tree!(5, "contains"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "end"), // identifier
                              ]),
                            ]),
                          ]),
                          tree!(37; [ // block
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "line"), // identifier
                                tree!(5, "setLine"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "end2d"), // identifier
                                  tree!(5, "y0"), // identifier
                                  tree!(5, "end2d"), // identifier
                                  tree!(5, "y1"), // identifier
                                ]),
                              ]),
                            ]),
                            tree!(29; [ // expression_statement
                              tree!(52; [ // method_invocation
                                tree!(5, "g2"), // identifier
                                tree!(5, "draw"), // identifier
                                tree!(35; [ // argument_list
                                  tree!(5, "line"), // identifier
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "String"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "label"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabel"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(50; [ // local_variable_declaration
                  tree!(14, "RectangleAnchor"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "anchor"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "marker"), // identifier
                      tree!(5, "getLabelAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
                tree!(40; [ // if_statement
                  tree!(41; [ // parenthesized_expression
                    tree!(42; [ // binary_expression
                      tree!(5, "label"), // identifier
                      tree!(43, "!="), // comparison_operator
                      tree!(44, "null"), // null_literal
                    ]),
                  ]),
                  tree!(37; [ // block
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "Font"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "labelFont"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "marker"), // identifier
                          tree!(5, "getLabelFont"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setFont"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "labelFont"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "g2"), // identifier
                        tree!(5, "setPaint"), // identifier
                        tree!(35; [ // argument_list
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getLabelPaint"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(50; [ // local_variable_declaration
                      tree!(14, "Point2D"), // type
                      tree!(22; [ // variable_declarator
                        tree!(5, "coordinates"), // identifier
                        tree!(23, "="), // affectation_operator
                        tree!(52; [ // method_invocation
                          tree!(5, "calculateRangeMarkerTextAnchorPoint"), // identifier
                          tree!(35; [ // argument_list
                            tree!(5, "g2"), // identifier
                            tree!(5, "orientation"), // identifier
                            tree!(5, "dataArea"), // identifier
                            tree!(5, "rect"), // identifier
                            tree!(52; [ // method_invocation
                              tree!(5, "marker"), // identifier
                              tree!(5, "getLabelOffset"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "marker"), // identifier
                              tree!(5, "getLabelOffsetType"), // identifier
                              tree!(35), // argument_list
                            ]),
                            tree!(5, "anchor"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(29; [ // expression_statement
                      tree!(52; [ // method_invocation
                        tree!(5, "TextUtilities"), // identifier
                        tree!(5, "drawAlignedString"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "label"), // identifier
                          tree!(5, "g2"), // identifier
                          tree!(51; [ // cast_expression
                            tree!(65; [ // floating_point_type
                              tree!(70, "float"), // float
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "coordinates"), // identifier
                              tree!(5, "getX"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                          tree!(51; [ // cast_expression
                            tree!(65; [ // floating_point_type
                              tree!(70, "float"), // float
                            ]),
                            tree!(52; [ // method_invocation
                              tree!(5, "coordinates"), // identifier
                              tree!(5, "getY"), // identifier
                              tree!(35), // argument_list
                            ]),
                          ]),
                          tree!(52; [ // method_invocation
                            tree!(5, "marker"), // identifier
                            tree!(5, "getLabelTextAnchor"), // identifier
                            tree!(35), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(29; [ // expression_statement
                  tree!(52; [ // method_invocation
                    tree!(5, "g2"), // identifier
                    tree!(5, "setComposite"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "savedComposite"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Calculates the (x, y) coordinates for drawing the label for a marker on\\n     * the range axis.\\n     *\\n     * @param g2  the graphics device.\\n     * @param orientation  the plot orientation.\\n     * @param dataArea  the data area.\\n     * @param markerArea  the rectangle surrounding the marker.\\n     * @param markerOffset  the marker offset.\\n     * @param labelOffsetType  the label offset type.\\n     * @param anchor  the label anchor.\\n     *\\n     * @return The coordinates for drawing the marker label.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "Point2D"), // type
        tree!(5, "calculateDomainMarkerTextAnchorPoint"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "markerArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleInsets"), // type
            tree!(5, "markerOffset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "LengthAdjustmentType"), // type
            tree!(5, "labelOffsetType"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleAnchor"), // type
            tree!(5, "anchor"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "anchorRect"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "anchorRect"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "markerOffset"), // identifier
                    tree!(5, "createAdjustedRectangle"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "markerArea"), // identifier
                      tree!(31; [ // field_access
                        tree!(5, "LengthAdjustmentType"), // identifier
                        tree!(5, "CONTRACT"), // identifier
                      ]),
                      tree!(5, "labelOffsetType"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "anchorRect"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "markerOffset"), // identifier
                      tree!(5, "createAdjustedRectangle"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "markerArea"), // identifier
                        tree!(5, "labelOffsetType"), // identifier
                        tree!(31; [ // field_access
                          tree!(5, "LengthAdjustmentType"), // identifier
                          tree!(5, "CONTRACT"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "RectangleAnchor"), // identifier
              tree!(5, "coordinates"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "anchorRect"), // identifier
                tree!(5, "anchor"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Calculates the (x, y) coordinates for drawing a marker label.\\n     *\\n     * @param g2  the graphics device.\\n     * @param orientation  the plot orientation.\\n     * @param dataArea  the data area.\\n     * @param markerArea  the rectangle surrounding the marker.\\n     * @param markerOffset  the marker offset.\\n     * @param labelOffsetType  the label offset type.\\n     * @param anchor  the label anchor.\\n     *\\n     * @return The coordinates for drawing the marker label.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "Point2D"), // type
        tree!(5, "calculateRangeMarkerTextAnchorPoint"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "markerArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleInsets"), // type
            tree!(5, "markerOffset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "LengthAdjustmentType"), // type
            tree!(5, "labelOffsetType"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "RectangleAnchor"), // type
            tree!(5, "anchor"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "anchorRect"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(31; [ // field_access
                  tree!(5, "PlotOrientation"), // identifier
                  tree!(5, "HORIZONTAL"), // identifier
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "anchorRect"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "markerOffset"), // identifier
                    tree!(5, "createAdjustedRectangle"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "markerArea"), // identifier
                      tree!(5, "labelOffsetType"), // identifier
                      tree!(31; [ // field_access
                        tree!(5, "LengthAdjustmentType"), // identifier
                        tree!(5, "CONTRACT"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(42; [ // binary_expression
                  tree!(5, "orientation"), // identifier
                  tree!(43, "=="), // comparison_operator
                  tree!(31; [ // field_access
                    tree!(5, "PlotOrientation"), // identifier
                    tree!(5, "VERTICAL"), // identifier
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "anchorRect"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(5, "markerOffset"), // identifier
                      tree!(5, "createAdjustedRectangle"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "markerArea"), // identifier
                        tree!(31; [ // field_access
                          tree!(5, "LengthAdjustmentType"), // identifier
                          tree!(5, "CONTRACT"), // identifier
                        ]),
                        tree!(5, "labelOffsetType"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "RectangleAnchor"), // identifier
              tree!(5, "coordinates"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "anchorRect"), // identifier
                tree!(5, "anchor"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a legend item for a series.  This default implementation will\\n     * return <code>null</code> if {@link #isSeriesVisible(int)} or\\n     * {@link #isSeriesVisibleInLegend(int)} returns <code>false</code>.\\n     *\\n     * @param datasetIndex  the dataset index (zero-based).\\n     * @param series  the series index (zero-based).\\n     *\\n     * @return The legend item (possibly <code>null</code>).\\n     *\\n     * @see #getLegendItems()\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "LegendItem"), // type
        tree!(5, "getLegendItem"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "datasetIndex"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "series"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryPlot"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "p"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getPlot"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "p"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(49, "// check that a legend item needs to be displayed..."), // line_comment
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(67; [ // unary_expression
                  tree!(68, "!"), // !
                  tree!(52; [ // method_invocation
                    tree!(5, "isSeriesVisible"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
                tree!(63, "||"), // logical_operator
                tree!(67; [ // unary_expression
                  tree!(68, "!"), // !
                  tree!(52; [ // method_invocation
                    tree!(5, "isSeriesVisibleInLegend"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDataset"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "dataset"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "p"), // identifier
                tree!(5, "getDataset"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "datasetIndex"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "label"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemLabelGenerator"), // identifier
                ]),
                tree!(5, "generateLabel"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "dataset"), // identifier
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "description"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(5, "label"), // identifier
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "toolTipText"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemToolTipGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "toolTipText"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemToolTipGenerator"), // identifier
                    ]),
                    tree!(5, "generateLabel"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "urlText"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemURLGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "urlText"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemURLGenerator"), // identifier
                    ]),
                    tree!(5, "generateLabel"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "series"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Shape"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "shape"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupLegendShape"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Paint"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "paint"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupSeriesPaint"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Paint"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "outlinePaint"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupSeriesOutlinePaint"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Stroke"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "outlineStroke"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupSeriesOutlineStroke"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "LegendItem"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "item"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "LegendItem"), // type
                tree!(35; [ // argument_list
                  tree!(5, "label"), // identifier
                  tree!(5, "description"), // identifier
                  tree!(5, "toolTipText"), // identifier
                  tree!(5, "urlText"), // identifier
                  tree!(5, "shape"), // identifier
                  tree!(5, "paint"), // identifier
                  tree!(5, "outlineStroke"), // identifier
                  tree!(5, "outlinePaint"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setLabelFont"), // identifier
              tree!(35; [ // argument_list
                tree!(52; [ // method_invocation
                  tree!(5, "lookupLegendTextFont"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "series"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Paint"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "labelPaint"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "lookupLegendTextPaint"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "series"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "labelPaint"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "item"), // identifier
                  tree!(5, "setLabelPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "labelPaint"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setSeriesKey"), // identifier
              tree!(35; [ // argument_list
                tree!(52; [ // method_invocation
                  tree!(5, "dataset"), // identifier
                  tree!(5, "getRowKey"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "series"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setSeriesIndex"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "series"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setDataset"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "dataset"), // identifier
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "item"), // identifier
              tree!(5, "setDatasetIndex"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "datasetIndex"), // identifier
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "item"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Tests this renderer for equality with another object.\\n     *\\n     * @param obj  the object.\\n     *\\n     * @return <code>true</code> or <code>false</code>.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "boolean"), // type
        tree!(5, "equals"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Object"), // type
            tree!(5, "obj"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "obj"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(32, "this"), // this
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(53, "true"), // true
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(5, "obj"), // identifier
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "AbstractCategoryItemRenderer"), // type
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "AbstractCategoryItemRenderer"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "that"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "AbstractCategoryItemRenderer"), // type
                tree!(5, "obj"), // identifier
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "itemLabelGeneratorList"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "itemLabelGeneratorList"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseItemLabelGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "baseItemLabelGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "toolTipGeneratorList"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "toolTipGeneratorList"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseToolTipGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "baseToolTipGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "urlGeneratorList"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "urlGeneratorList"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseURLGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "baseURLGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemLabelGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "legendItemLabelGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemToolTipGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "legendItemToolTipGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "legendItemURLGenerator"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "legendItemURLGenerator"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "backgroundAnnotations"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "backgroundAnnotations"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "ObjectUtilities"), // identifier
                  tree!(5, "equal"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "foregroundAnnotations"), // identifier
                    ]),
                    tree!(31; [ // field_access
                      tree!(5, "that"), // identifier
                      tree!(5, "foregroundAnnotations"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(71, "super"), // super
              tree!(5, "equals"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "obj"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a hash code for the renderer.\\n     *\\n     * @return The hash code.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "int"), // type
        tree!(5, "hashCode"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(71, "super"), // super
                tree!(5, "hashCode"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the drawing supplier from the plot.\\n     *\\n     * @return The drawing supplier (possibly <code>null</code>).\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "DrawingSupplier"), // type
        tree!(5, "getDrawingSupplier"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "DrawingSupplier"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryPlot"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "cp"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getPlot"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "cp"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "cp"), // identifier
                    tree!(5, "getDrawingSupplier"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Considers the current (x, y) coordinate and updates the crosshair point\\n     * if it meets the criteria (usually means the (x, y) coordinate is the\\n     * closest to the anchor point so far).\\n     *\\n     * @param crosshairState  the crosshair state (<code>null</code> permitted,\\n     *                        but the method does nothing in that case).\\n     * @param rowKey  the row key.\\n     * @param columnKey  the column key.\\n     * @param value  the data value.\\n     * @param datasetIndex  the dataset index.\\n     * @param transX  the x-value translated to Java2D space.\\n     * @param transY  the y-value translated to Java2D space.\\n     * @param orientation  the plot orientation (<code>null</code> not\\n     *                     permitted).\\n     *\\n     * @since 1.0.11\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "updateCrosshairValues"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryCrosshairState"), // type
            tree!(5, "crosshairState"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "rowKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Comparable"), // type
            tree!(5, "columnKey"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "value"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "datasetIndex"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "transX"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "transY"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "orientation"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'orientation' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "crosshairState"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "plot"), // identifier
                    ]),
                    tree!(5, "isRangeCrosshairLockedOnData"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(49, "// both axes"), // line_comment
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "crosshairState"), // identifier
                      tree!(5, "updateCrosshairPoint"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "rowKey"), // identifier
                        tree!(5, "columnKey"), // identifier
                        tree!(5, "value"), // identifier
                        tree!(5, "datasetIndex"), // identifier
                        tree!(5, "transX"), // identifier
                        tree!(5, "transY"), // identifier
                        tree!(5, "orientation"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(52; [ // method_invocation
                      tree!(5, "crosshairState"), // identifier
                      tree!(5, "updateCrosshairX"), // identifier
                      tree!(35; [ // argument_list
                        tree!(5, "rowKey"), // identifier
                        tree!(5, "columnKey"), // identifier
                        tree!(5, "datasetIndex"), // identifier
                        tree!(5, "transX"), // identifier
                        tree!(5, "orientation"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws an item label.\\n     *\\n     * @param g2  the graphics device.\\n     * @param orientation  the orientation.\\n     * @param dataset  the dataset.\\n     * @param row  the row.\\n     * @param column  the column.\\n     * @param selected  is the item selected?\\n     * @param x  the x coordinate (in Java2D space).\\n     * @param y  the y coordinate (in Java2D space).\\n     * @param negative  indicates a negative value (which affects the item\\n     *                  label position).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawItemLabel"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotOrientation"), // type
            tree!(5, "orientation"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "x"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "y"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "negative"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemLabelGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getItemLabelGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "Font"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "labelFont"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "getItemLabelFont"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                      tree!(5, "selected"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Paint"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "paint"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "getItemLabelPaint"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                      tree!(5, "selected"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setFont"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "labelFont"), // identifier
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "g2"), // identifier
                  tree!(5, "setPaint"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "paint"), // identifier
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "String"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "label"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "generator"), // identifier
                    tree!(5, "generateLabel"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "ItemLabelPosition"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "position"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(44, "null"), // null_literal
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(67; [ // unary_expression
                    tree!(68, "!"), // !
                    tree!(5, "negative"), // identifier
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "position"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "getPositiveItemLabelPosition"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "row"), // identifier
                          tree!(5, "column"), // identifier
                          tree!(5, "selected"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "position"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(52; [ // method_invocation
                        tree!(5, "getNegativeItemLabelPosition"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "row"), // identifier
                          tree!(5, "column"), // identifier
                          tree!(5, "selected"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(14, "Point2D"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "anchorPoint"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "calculateLabelAnchorPoint"), // identifier
                    tree!(35; [ // argument_list
                      tree!(52; [ // method_invocation
                        tree!(5, "position"), // identifier
                        tree!(5, "getItemLabelAnchor"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "x"), // identifier
                      tree!(5, "y"), // identifier
                      tree!(5, "orientation"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "TextUtilities"), // identifier
                  tree!(5, "drawRotatedString"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "label"), // identifier
                    tree!(5, "g2"), // identifier
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "anchorPoint"), // identifier
                        tree!(5, "getX"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(51; [ // cast_expression
                      tree!(65; [ // floating_point_type
                        tree!(70, "float"), // float
                      ]),
                      tree!(52; [ // method_invocation
                        tree!(5, "anchorPoint"), // identifier
                        tree!(5, "getY"), // identifier
                        tree!(35), // argument_list
                      ]),
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "position"), // identifier
                      tree!(5, "getTextAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "position"), // identifier
                      tree!(5, "getAngle"), // identifier
                      tree!(35), // argument_list
                    ]),
                    tree!(52; [ // method_invocation
                      tree!(5, "position"), // identifier
                      tree!(5, "getRotationAnchor"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Draws all the annotations for the specified layer.\\n     *\\n     * @param g2  the graphics device.\\n     * @param dataArea  the data area.\\n     * @param domainAxis  the domain axis.\\n     * @param rangeAxis  the range axis.\\n     * @param layer  the layer.\\n     * @param info  the plot rendering info.\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "drawAnnotations"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Layer"), // type
            tree!(5, "layer"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "PlotRenderingInfo"), // type
            tree!(5, "info"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Iterator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "iterator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "layer"), // identifier
                tree!(5, "equals"), // identifier
                tree!(35; [ // argument_list
                  tree!(31; [ // field_access
                    tree!(5, "Layer"), // identifier
                    tree!(5, "FOREGROUND"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "iterator"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "foregroundAnnotations"), // identifier
                    ]),
                    tree!(5, "iterator"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
            tree!(40; [ // if_statement
              tree!(41; [ // parenthesized_expression
                tree!(52; [ // method_invocation
                  tree!(5, "layer"), // identifier
                  tree!(5, "equals"), // identifier
                  tree!(35; [ // argument_list
                    tree!(31; [ // field_access
                      tree!(5, "Layer"), // identifier
                      tree!(5, "BACKGROUND"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(29; [ // expression_statement
                  tree!(30; [ // assignment_expression
                    tree!(5, "iterator"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "backgroundAnnotations"), // identifier
                      ]),
                      tree!(5, "iterator"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(37; [ // block
                tree!(49, "// should not get here"), // line_comment
                tree!(45; [ // throw_statement
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "RuntimeException"), // type
                    tree!(35; [ // argument_list
                      tree!(46; [ // string_literal
                        tree!(47, "\""), // "
                        tree!(48, "Unknown layer."), // string_fragment
                        tree!(47, "\""), // "
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(72; [ // while_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(5, "iterator"), // identifier
                tree!(5, "hasNext"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(14, "CategoryAnnotation"), // type
                tree!(22; [ // variable_declarator
                  tree!(5, "annotation"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategoryAnnotation"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "iterator"), // identifier
                      tree!(5, "next"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(29; [ // expression_statement
                tree!(52; [ // method_invocation
                  tree!(5, "annotation"), // identifier
                  tree!(5, "draw"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "g2"), // identifier
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "plot"), // identifier
                    ]),
                    tree!(5, "dataArea"), // identifier
                    tree!(5, "domainAxis"), // identifier
                    tree!(5, "rangeAxis"), // identifier
                    tree!(24, "0"), // decimal_integer_literal
                    tree!(5, "info"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns an independent copy of the renderer.  The <code>plot</code>\\n     * reference is shallow copied.\\n     *\\n     * @return A clone.\\n     *\\n     * @throws CloneNotSupportedException  can be thrown if one of the objects\\n     *         belonging to the renderer does not support cloning (for example,\\n     *         an item label generator).\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Object"), // type
        tree!(5, "clone"), // identifier
        tree!(27), // formal_parameters
        tree!(73; [ // throws
          tree!(73, "throws"), // throws
          tree!(14, "CloneNotSupportedException"), // type
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "AbstractCategoryItemRenderer"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "clone"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(51; [ // cast_expression
                tree!(14, "AbstractCategoryItemRenderer"), // type
                tree!(52; [ // method_invocation
                  tree!(71, "super"), // super
                  tree!(5, "clone"), // identifier
                  tree!(35), // argument_list
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "itemLabelGeneratorList"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "itemLabelGeneratorList"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ObjectList"), // type
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "itemLabelGeneratorList"), // identifier
                      ]),
                      tree!(5, "clone"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "baseItemLabelGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseItemLabelGenerator"), // identifier
                    ]),
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "PublicCloneable"), // type
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "PublicCloneable"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "pc"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "PublicCloneable"), // type
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "baseItemLabelGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(31; [ // field_access
                        tree!(5, "clone"), // identifier
                        tree!(5, "baseItemLabelGenerator"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryItemLabelGenerator"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "pc"), // identifier
                          tree!(5, "clone"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(45; [ // throw_statement
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "CloneNotSupportedException"), // type
                      tree!(35; [ // argument_list
                        tree!(46; [ // string_literal
                          tree!(47, "\""), // "
                          tree!(48, "ItemLabelGenerator not cloneable."), // string_fragment
                          tree!(47, "\""), // "
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "toolTipGeneratorList"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "toolTipGeneratorList"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ObjectList"), // type
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "toolTipGeneratorList"), // identifier
                      ]),
                      tree!(5, "clone"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "baseToolTipGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseToolTipGenerator"), // identifier
                    ]),
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "PublicCloneable"), // type
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "PublicCloneable"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "pc"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "PublicCloneable"), // type
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "baseToolTipGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(31; [ // field_access
                        tree!(5, "clone"), // identifier
                        tree!(5, "baseToolTipGenerator"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryToolTipGenerator"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "pc"), // identifier
                          tree!(5, "clone"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(45; [ // throw_statement
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "CloneNotSupportedException"), // type
                      tree!(35; [ // argument_list
                        tree!(46; [ // string_literal
                          tree!(47, "\""), // "
                          tree!(48, "Base tool tip generator not cloneable."), // string_fragment
                          tree!(47, "\""), // "
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "urlGeneratorList"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "urlGeneratorList"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "ObjectList"), // type
                    tree!(52; [ // method_invocation
                      tree!(31; [ // field_access
                        tree!(32, "this"), // this
                        tree!(5, "urlGeneratorList"), // identifier
                      ]),
                      tree!(5, "clone"), // identifier
                      tree!(35), // argument_list
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "baseURLGenerator"), // identifier
                ]),
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(61; [ // instanceof_expression
                    tree!(31; [ // field_access
                      tree!(32, "this"), // this
                      tree!(5, "baseURLGenerator"), // identifier
                    ]),
                    tree!(62, "instanceof"), // instanceof
                    tree!(14, "PublicCloneable"), // type
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(50; [ // local_variable_declaration
                    tree!(14, "PublicCloneable"), // type
                    tree!(22; [ // variable_declarator
                      tree!(5, "pc"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "PublicCloneable"), // type
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "baseURLGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(31; [ // field_access
                        tree!(5, "clone"), // identifier
                        tree!(5, "baseURLGenerator"), // identifier
                      ]),
                      tree!(23, "="), // affectation_operator
                      tree!(51; [ // cast_expression
                        tree!(14, "CategoryURLGenerator"), // type
                        tree!(52; [ // method_invocation
                          tree!(5, "pc"), // identifier
                          tree!(5, "clone"), // identifier
                          tree!(35), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(45; [ // throw_statement
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(14, "CloneNotSupportedException"), // type
                      tree!(35; [ // argument_list
                        tree!(46; [ // string_literal
                          tree!(47, "\""), // "
                          tree!(48, "Base item URL generator not cloneable."), // string_fragment
                          tree!(47, "\""), // "
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemLabelGenerator"), // identifier
                ]),
                tree!(62, "instanceof"), // instanceof
                tree!(14, "PublicCloneable"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "legendItemLabelGenerator"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategorySeriesLabelGenerator"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "ObjectUtilities"), // identifier
                      tree!(5, "clone"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "legendItemLabelGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemToolTipGenerator"), // identifier
                ]),
                tree!(62, "instanceof"), // instanceof
                tree!(14, "PublicCloneable"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "legendItemToolTipGenerator"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategorySeriesLabelGenerator"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "ObjectUtilities"), // identifier
                      tree!(5, "clone"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "legendItemToolTipGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(61; [ // instanceof_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "legendItemURLGenerator"), // identifier
                ]),
                tree!(62, "instanceof"), // instanceof
                tree!(14, "PublicCloneable"), // type
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(31; [ // field_access
                    tree!(5, "clone"), // identifier
                    tree!(5, "legendItemURLGenerator"), // identifier
                  ]),
                  tree!(23, "="), // affectation_operator
                  tree!(51; [ // cast_expression
                    tree!(14, "CategorySeriesLabelGenerator"), // type
                    tree!(52; [ // method_invocation
                      tree!(5, "ObjectUtilities"), // identifier
                      tree!(5, "clone"), // identifier
                      tree!(35; [ // argument_list
                        tree!(31; [ // field_access
                          tree!(32, "this"), // this
                          tree!(5, "legendItemURLGenerator"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "clone"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the domain axis that is used for the specified dataset.\\n     *\\n     * @param plot  the plot (<code>null</code> not permitted).\\n     * @param dataset  the dataset (<code>null</code> not permitted).\\n     *\\n     * @return A domain axis.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "CategoryAxis"), // type
        tree!(5, "getDomainAxis"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "datasetIndex"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "indexOf"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "dataset"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "plot"), // identifier
              tree!(5, "getDomainAxisForDataset"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "datasetIndex"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a range axis for a plot.\\n     *\\n     * @param plot  the plot.\\n     * @param index  the axis index.\\n     *\\n     * @return A range axis.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "ValueAxis"), // type
        tree!(5, "getRangeAxis"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "index"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "ValueAxis"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "plot"), // identifier
                tree!(5, "getRangeAxis"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "index"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "result"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getRangeAxis"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a (possibly empty) collection of legend items for the series\\n     * that this renderer is responsible for drawing.\\n     *\\n     * @return The legend item collection (never <code>null</code>).\\n     *\\n     * @see #getLegendItem(int, int)\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "LegendItemCollection"), // type
        tree!(5, "getLegendItems"), // identifier
        tree!(27), // formal_parameters
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "LegendItemCollection"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "LegendItemCollection"), // type
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "plot"), // identifier
                ]),
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(5, "result"), // identifier
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "index"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "plot"), // identifier
                ]),
                tree!(5, "getIndexOf"), // identifier
                tree!(35; [ // argument_list
                  tree!(32, "this"), // this
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryDataset"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "dataset"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(31; [ // field_access
                  tree!(32, "this"), // this
                  tree!(5, "plot"), // identifier
                ]),
                tree!(5, "getDataset"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "index"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "dataset"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(5, "result"), // identifier
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "int"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "seriesCount"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getRowCount"), // identifier
                tree!(35), // argument_list
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(52; [ // method_invocation
                tree!(52; [ // method_invocation
                  tree!(5, "plot"), // identifier
                  tree!(5, "getRowRenderingOrder"), // identifier
                  tree!(35), // argument_list
                ]),
                tree!(5, "equals"), // identifier
                tree!(35; [ // argument_list
                  tree!(31; [ // field_access
                    tree!(5, "SortOrder"), // identifier
                    tree!(5, "ASCENDING"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(57; [ // for_statement
                tree!(50; [ // local_variable_declaration
                  tree!(14, "int"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "i"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(24, "0"), // decimal_integer_literal
                  ]),
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "i"), // identifier
                  tree!(43, "<"), // comparison_operator
                  tree!(5, "seriesCount"), // identifier
                ]),
                tree!(58; [ // update_expression
                  tree!(5, "i"), // identifier
                  tree!(59, "++"), // increment_operator
                ]),
                tree!(37; [ // block
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(52; [ // method_invocation
                        tree!(5, "isSeriesVisibleInLegend"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "i"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(50; [ // local_variable_declaration
                        tree!(14, "LegendItem"), // type
                        tree!(22; [ // variable_declarator
                          tree!(5, "item"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "getLegendItem"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "index"), // identifier
                              tree!(5, "i"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(40; [ // if_statement
                        tree!(41; [ // parenthesized_expression
                          tree!(42; [ // binary_expression
                            tree!(5, "item"), // identifier
                            tree!(43, "!="), // comparison_operator
                            tree!(44, "null"), // null_literal
                          ]),
                        ]),
                        tree!(37; [ // block
                          tree!(29; [ // expression_statement
                            tree!(52; [ // method_invocation
                              tree!(5, "result"), // identifier
                              tree!(5, "add"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "item"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(57; [ // for_statement
                tree!(50; [ // local_variable_declaration
                  tree!(14, "int"), // type
                  tree!(22; [ // variable_declarator
                    tree!(5, "i"), // identifier
                    tree!(23, "="), // affectation_operator
                    tree!(42; [ // binary_expression
                      tree!(5, "seriesCount"), // identifier
                      tree!(69, "-"), // arithmetic_operator
                      tree!(24, "1"), // decimal_integer_literal
                    ]),
                  ]),
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "i"), // identifier
                  tree!(43, ">="), // comparison_operator
                  tree!(24, "0"), // decimal_integer_literal
                ]),
                tree!(58; [ // update_expression
                  tree!(5, "i"), // identifier
                  tree!(59, "--"), // increment_operator
                ]),
                tree!(37; [ // block
                  tree!(40; [ // if_statement
                    tree!(41; [ // parenthesized_expression
                      tree!(52; [ // method_invocation
                        tree!(5, "isSeriesVisibleInLegend"), // identifier
                        tree!(35; [ // argument_list
                          tree!(5, "i"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(37; [ // block
                      tree!(50; [ // local_variable_declaration
                        tree!(14, "LegendItem"), // type
                        tree!(22; [ // variable_declarator
                          tree!(5, "item"), // identifier
                          tree!(23, "="), // affectation_operator
                          tree!(52; [ // method_invocation
                            tree!(5, "getLegendItem"), // identifier
                            tree!(35; [ // argument_list
                              tree!(5, "index"), // identifier
                              tree!(5, "i"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(40; [ // if_statement
                        tree!(41; [ // parenthesized_expression
                          tree!(42; [ // binary_expression
                            tree!(5, "item"), // identifier
                            tree!(43, "!="), // comparison_operator
                            tree!(44, "null"), // null_literal
                          ]),
                        ]),
                        tree!(37; [ // block
                          tree!(29; [ // expression_statement
                            tree!(52; [ // method_invocation
                              tree!(5, "result"), // identifier
                              tree!(5, "add"), // identifier
                              tree!(35; [ // argument_list
                                tree!(5, "item"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Adds an entity with the specified hotspot.\\n     *\\n     * @param entities  the entity collection.\\n     * @param hotspot  the hotspot (<code>null</code> not permitted).\\n     * @param dataset  the dataset.\\n     * @param row  the row index.\\n     * @param column  the column index.\\n     * @param selected  is the item selected?\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addEntity"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "EntityCollection"), // type
            tree!(5, "entities"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Shape"), // type
            tree!(5, "hotspot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "hotspot"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(45; [ // throw_statement
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(14, "IllegalArgumentException"), // type
                  tree!(35; [ // argument_list
                    tree!(46; [ // string_literal
                      tree!(47, "\""), // "
                      tree!(48, "Null 'hotspot' argument."), // string_fragment
                      tree!(47, "\""), // "
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "addEntity"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "entities"), // identifier
                tree!(5, "hotspot"), // identifier
                tree!(5, "dataset"), // identifier
                tree!(5, "row"), // identifier
                tree!(5, "column"), // identifier
                tree!(5, "selected"), // identifier
                tree!(74, "0.0"), // decimal_floating_point_literal
                tree!(74, "0.0"), // decimal_floating_point_literal
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Adds an entity to the collection.\\n     *\\n     * @param entities  the entity collection being populated.\\n     * @param hotspot  the entity area (if <code>null</code> a default will be\\n     *              used).\\n     * @param dataset  the dataset.\\n     * @param row  the series.\\n     * @param column  the item.\\n     * @param selected  is the item selected?\\n     * @param entityX  the entity's center x-coordinate in user space (only\\n     *                 used if <code>area</code> is <code>null</code>).\\n     * @param entityY  the entity's center y-coordinate in user space (only\\n     *                 used if <code>area</code> is <code>null</code>).\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "protected"), // visibility
        ]),
        tree!(14, "void"), // type
        tree!(5, "addEntity"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "EntityCollection"), // type
            tree!(5, "entities"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Shape"), // type
            tree!(5, "hotspot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "entityX"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "entityY"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(67; [ // unary_expression
                tree!(68, "!"), // !
                tree!(52; [ // method_invocation
                  tree!(5, "getItemCreateEntity"), // identifier
                  tree!(35; [ // argument_list
                    tree!(5, "row"), // identifier
                    tree!(5, "column"), // identifier
                    tree!(5, "selected"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(37; [ // block
              tree!(38), // return_statement
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Shape"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "s"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(5, "hotspot"), // identifier
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "hotspot"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "r"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "getDefaultEntityRadius"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
              tree!(50; [ // local_variable_declaration
                tree!(65; [ // floating_point_type
                  tree!(66, "double"), // double
                ]),
                tree!(22; [ // variable_declarator
                  tree!(5, "w"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(42; [ // binary_expression
                    tree!(5, "r"), // identifier
                    tree!(69, "*"), // arithmetic_operator
                    tree!(24, "2"), // decimal_integer_literal
                  ]),
                ]),
              ]),
              tree!(40; [ // if_statement
                tree!(41; [ // parenthesized_expression
                  tree!(42; [ // binary_expression
                    tree!(52; [ // method_invocation
                      tree!(52; [ // method_invocation
                        tree!(5, "getPlot"), // identifier
                        tree!(35), // argument_list
                      ]),
                      tree!(5, "getOrientation"), // identifier
                      tree!(35), // argument_list
                    ]),
                    tree!(43, "=="), // comparison_operator
                    tree!(31; [ // field_access
                      tree!(5, "PlotOrientation"), // identifier
                      tree!(5, "VERTICAL"), // identifier
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "s"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Ellipse2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(42; [ // binary_expression
                            tree!(5, "entityX"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(42; [ // binary_expression
                            tree!(5, "entityY"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(5, "w"), // identifier
                          tree!(5, "w"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(37; [ // block
                  tree!(29; [ // expression_statement
                    tree!(30; [ // assignment_expression
                      tree!(5, "s"), // identifier
                      tree!(23, "="), // affectation_operator
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(14, "Ellipse2D.Double"), // type
                        tree!(35; [ // argument_list
                          tree!(42; [ // binary_expression
                            tree!(5, "entityY"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(42; [ // binary_expression
                            tree!(5, "entityX"), // identifier
                            tree!(69, "-"), // arithmetic_operator
                            tree!(5, "r"), // identifier
                          ]),
                          tree!(5, "w"), // identifier
                          tree!(5, "w"), // identifier
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "tip"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryToolTipGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "generator"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getToolTipGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "generator"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "tip"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "generator"), // identifier
                    tree!(5, "generateToolTip"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "String"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "url"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(44, "null"), // null_literal
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryURLGenerator"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "urlster"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "getURLGenerator"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "urlster"), // identifier
                tree!(43, "!="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "url"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(52; [ // method_invocation
                    tree!(5, "urlster"), // identifier
                    tree!(5, "generateURL"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "dataset"), // identifier
                      tree!(5, "row"), // identifier
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "CategoryItemEntity"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "entity"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(14, "CategoryItemEntity"), // type
                tree!(35; [ // argument_list
                  tree!(5, "s"), // identifier
                  tree!(5, "tip"), // identifier
                  tree!(5, "url"), // identifier
                  tree!(5, "dataset"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getRowKey"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "row"), // identifier
                    ]),
                  ]),
                  tree!(52; [ // method_invocation
                    tree!(5, "dataset"), // identifier
                    tree!(5, "getColumnKey"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "column"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "entities"), // identifier
              tree!(5, "add"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "entity"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "        \\n     * Returns a shape that can be used for hit testing on a data item drawn\\n     * by the renderer.\\n     *\\n     * @param g2  the graphics device.\\n     * @param dataArea  the area within which the data is being rendered.\\n     * @param plot  the plot (can be used to obtain standard color\\n     *              information etc).\\n     * @param domainAxis  the domain axis.\\n     * @param rangeAxis  the range axis.\\n     * @param dataset  the dataset.\\n     * @param row  the row index (zero-based).\\n     * @param column  the column index (zero-based).\\n     * @param selected  is the item selected?\\n     *\\n     * @return A shape equal to the hot spot for a data item.\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Shape"), // type
        tree!(5, "createHotSpotShape"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemRendererState"), // type
            tree!(5, "state"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(45; [ // throw_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(14, "RuntimeException"), // type
              tree!(35; [ // argument_list
                tree!(46; [ // string_literal
                  tree!(47, "\""), // "
                  tree!(48, "Not implemented."), // string_fragment
                  tree!(47, "\""), // "
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the rectangular bounds for the hot spot for an item drawn by\\n     * this renderer.  This is intended to provide a quick test for\\n     * eliminating data points before more accurate testing against the\\n     * shape returned by createHotSpotShape().\\n     *\\n     * @param g2\\n     * @param dataArea\\n     * @param plot\\n     * @param domainAxis\\n     * @param rangeAxis\\n     * @param dataset\\n     * @param row\\n     * @param column\\n     * @param selected\\n     * @param result\\n     * @return\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "Rectangle2D"), // type
        tree!(5, "createHotSpotBounds"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemRendererState"), // type
            tree!(5, "state"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "result"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "result"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(29; [ // expression_statement
                tree!(30; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(23, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(14, "Rectangle"), // type
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Comparable"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "key"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getColumnKey"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "column"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(14, "Number"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "y"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "dataset"), // identifier
                tree!(5, "getValue"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "y"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(44, "null"), // null_literal
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(22; [ // variable_declarator
              tree!(5, "xx"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "domainAxis"), // identifier
                tree!(5, "getCategoryMiddle"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "key"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getCategoriesForAxis"), // identifier
                    tree!(35; [ // argument_list
                      tree!(5, "domainAxis"), // identifier
                    ]),
                  ]),
                  tree!(5, "dataArea"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getDomainAxisEdge"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(50; [ // local_variable_declaration
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(22; [ // variable_declarator
              tree!(5, "yy"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "rangeAxis"), // identifier
                tree!(5, "valueToJava2D"), // identifier
                tree!(35; [ // argument_list
                  tree!(52; [ // method_invocation
                    tree!(5, "y"), // identifier
                    tree!(5, "doubleValue"), // identifier
                    tree!(35), // argument_list
                  ]),
                  tree!(5, "dataArea"), // identifier
                  tree!(52; [ // method_invocation
                    tree!(5, "plot"), // identifier
                    tree!(5, "getRangeAxisEdge"), // identifier
                    tree!(35), // argument_list
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(29; [ // expression_statement
            tree!(52; [ // method_invocation
              tree!(5, "result"), // identifier
              tree!(5, "setRect"), // identifier
              tree!(35; [ // argument_list
                tree!(42; [ // binary_expression
                  tree!(5, "xx"), // identifier
                  tree!(69, "-"), // arithmetic_operator
                  tree!(24, "2"), // decimal_integer_literal
                ]),
                tree!(42; [ // binary_expression
                  tree!(5, "yy"), // identifier
                  tree!(69, "-"), // arithmetic_operator
                  tree!(24, "2"), // decimal_integer_literal
                ]),
                tree!(24, "4"), // decimal_integer_literal
                tree!(24, "4"), // decimal_integer_literal
              ]),
            ]),
          ]),
          tree!(38; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns <code>true</code> if the specified point (xx, yy) in Java2D\\n     * space falls within the \"hot spot\" for the specified data item, and\\n     * <code>false</code> otherwise.\\n     *\\n     * @param xx\\n     * @param yy\\n     * @param g2\\n     * @param dataArea\\n     * @param plot\\n     * @param domainAxis\\n     * @param rangeAxis\\n     * @param dataset\\n     * @param row\\n     * @param column\\n     * @param selected\\n     *\\n     * @return\\n     *\\n     * @since 1.2.0\\n     */"), // block_comment
      tree!(36; [ // method_declaration
        tree!(8; [ // modifiers
          tree!(9, "public"), // visibility
        ]),
        tree!(14, "boolean"), // type
        tree!(5, "hitTest"), // identifier
        tree!(27; [ // formal_parameters
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "xx"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(65; [ // floating_point_type
              tree!(66, "double"), // double
            ]),
            tree!(5, "yy"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Graphics2D"), // type
            tree!(5, "g2"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "Rectangle2D"), // type
            tree!(5, "dataArea"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryPlot"), // type
            tree!(5, "plot"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryAxis"), // type
            tree!(5, "domainAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "ValueAxis"), // type
            tree!(5, "rangeAxis"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryDataset"), // type
            tree!(5, "dataset"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "row"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "int"), // type
            tree!(5, "column"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "boolean"), // type
            tree!(5, "selected"), // identifier
          ]),
          tree!(39; [ // formal_parameter
            tree!(14, "CategoryItemRendererState"), // type
            tree!(5, "state"), // identifier
          ]),
        ]),
        tree!(37; [ // block
          tree!(50; [ // local_variable_declaration
            tree!(14, "Rectangle2D"), // type
            tree!(22; [ // variable_declarator
              tree!(5, "bounds"), // identifier
              tree!(23, "="), // affectation_operator
              tree!(52; [ // method_invocation
                tree!(5, "createHotSpotBounds"), // identifier
                tree!(35; [ // argument_list
                  tree!(5, "g2"), // identifier
                  tree!(5, "dataArea"), // identifier
                  tree!(5, "plot"), // identifier
                  tree!(5, "domainAxis"), // identifier
                  tree!(5, "rangeAxis"), // identifier
                  tree!(5, "dataset"), // identifier
                  tree!(5, "row"), // identifier
                  tree!(5, "column"), // identifier
                  tree!(5, "selected"), // identifier
                  tree!(5, "state"), // identifier
                  tree!(44, "null"), // null_literal
                ]),
              ]),
            ]),
          ]),
          tree!(40; [ // if_statement
            tree!(41; [ // parenthesized_expression
              tree!(42; [ // binary_expression
                tree!(5, "bounds"), // identifier
                tree!(43, "=="), // comparison_operator
                tree!(44, "null"), // null_literal
              ]),
            ]),
            tree!(37; [ // block
              tree!(38; [ // return_statement
                tree!(64, "false"), // false
              ]),
            ]),
          ]),
          tree!(49, "// FIXME:  if the following test passes, we should then do the more"), // line_comment
          tree!(49, "// expensive test against the hotSpotShape"), // line_comment
          tree!(38; [ // return_statement
            tree!(52; [ // method_invocation
              tree!(5, "bounds"), // identifier
              tree!(5, "contains"), // identifier
              tree!(35; [ // argument_list
                tree!(5, "xx"), // identifier
                tree!(5, "yy"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
    ]),
  ]),
]);

    (src_tr, dst_tr)

}


pub(crate) fn example_csv_11() -> (SimpleTree<u8>, SimpleTree<u8>) {
    let src_tr = tree!(1; [ // program
  tree!(2, "\\n * Licensed to the Apache Software Foundation (ASF) under one or more\\n * contributor license agreements.  See the NOTICE file distributed with\\n * this work for additional information regarding copyright ownership.\\n * The ASF licenses this file to You under the Apache License, Version 2.0\\n * (the \"License\"); you may not use this file except in compliance with\\n * the License.  You may obtain a copy of the License at\\n *\\n *      http://www.apache.org/licenses/LICENSE-2.0\\n *\\n * Unless required by applicable law or agreed to in writing, software\\n * distributed under the License is distributed on an \"AS IS\" BASIS,\\n * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\\n * See the License for the specific language governing permissions and\\n * limitations under the License.\\n */"), // block_comment
  tree!(3; [ // package_declaration
    tree!(4, "package"), // package
    tree!(5, "org.apache.commons.csv"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.Closeable"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.File"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.FileInputStream"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.FileReader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.IOException"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.InputStreamReader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.Reader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.StringReader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.net.URL"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.nio.charset.Charset"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.ArrayList"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Arrays"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Collection"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Iterator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.LinkedHashMap"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.List"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Map"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.NoSuchElementException"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(7, "static"), // static
    tree!(5, "org.apache.commons.csv.Token.Type"), // identifier
    tree!(8; [ // asterisk
      tree!(9, "*"), // arithmetic_operator
    ]),
  ]),
  tree!(2, "\\n * Parses CSV files according to the specified format.\\n *\\n * Because CSV appears in many different dialects, the parser supports many formats by allowing the\\n * specification of a {@link CSVFormat}.\\n *\\n * The parser works record wise. It is not possible to go back, once a record has been parsed from the input stream.\\n *\\n * <h2>Creating instances</h2>\\n * <p>\\n * There are several static factory methods that can be used to create instances for various types of resources:\\n * </p>\\n * <ul>\\n *     <li>{@link #parse(java.io.File, Charset, CSVFormat)}</li>\\n *     <li>{@link #parse(String, CSVFormat)}</li>\\n *     <li>{@link #parse(java.net.URL, java.nio.charset.Charset, CSVFormat)}</li>\\n * </ul>\\n * <p>\\n * Alternatively parsers can also be created by passing a {@link Reader} directly to the sole constructor.\\n *\\n * For those who like fluent APIs, parsers can be created using {@link CSVFormat#parse(java.io.Reader)} as a shortcut:\\n * </p>\\n * <pre>\\n * for(CSVRecord record : CSVFormat.EXCEL.parse(in)) {\\n *     ...\\n * }\\n * </pre>\\n *\\n * <h2>Parsing record wise</h2>\\n * <p>\\n * To parse a CSV input from a file, you write:\\n * </p>\\n *\\n * <pre>\\n * File csvData = new File(&quot;/path/to/csv&quot;);\\n * CSVParser parser = CSVParser.parse(csvData, CSVFormat.RFC4180);\\n * for (CSVRecord csvRecord : parser) {\\n *     ...\\n * }\\n * </pre>\\n *\\n * <p>\\n * This will read the parse the contents of the file using the\\n * <a href=\"http://tools.ietf.org/html/rfc4180\" target=\"_blank\">RFC 4180</a> format.\\n * </p>\\n *\\n * <p>\\n * To parse CSV input in a format like Excel, you write:\\n * </p>\\n *\\n * <pre>\\n * CSVParser parser = CSVParser.parse(csvData, CSVFormat.EXCEL);\\n * for (CSVRecord csvRecord : parser) {\\n *     ...\\n * }\\n * </pre>\\n *\\n * <p>\\n * If the predefined formats don't match the format at hands, custom formats can be defined. More information about\\n * customising CSVFormats is available in {@link CSVFormat CSVFormat JavaDoc}.\\n * </p>\\n *\\n * <h2>Parsing into memory</h2>\\n * <p>\\n * If parsing record wise is not desired, the contents of the input can be read completely into memory.\\n * </p>\\n *\\n * <pre>\\n * Reader in = new StringReader(&quot;a;b\\nc;d&quot;);\\n * CSVParser parser = new CSVParser(in, CSVFormat.EXCEL);\\n * List&lt;CSVRecord&gt; list = parser.getRecords();\\n * </pre>\\n *\\n * <p>\\n * There are two constraints that have to be kept in mind:\\n * </p>\\n *\\n * <ol>\\n *     <li>Parsing into memory starts at the current position of the parser. If you have already parsed records from\\n *     the input, those records will not end up in the in memory representation of your CSV data.</li>\\n *     <li>Parsing into memory may consume a lot of system resources depending on the input. For example if you're\\n *     parsing a 150MB file of CSV data the contents will be read completely into memory.</li>\\n * </ol>\\n *\\n * <h2>Notes</h2>\\n * <p>\\n * Internal parser state is completely covered by the format and the reader-state.\\n * </p>\\n *\\n * @version $Id$\\n *\\n * @see <a href=\"package-summary.html\">package documentation for more details</a>\\n */"), // block_comment
  tree!(10; [ // type_declaration
    tree!(11; [ // modifiers
      tree!(12, "public"), // visibility
      tree!(13, "final"), // final
    ]),
    tree!(14, "class"), // type_keyword
    tree!(5, "CSVParser"), // identifier
    tree!(15; [ // super_interfaces
      tree!(16, "implements"), // implements
      tree!(17; [ // type_list
        tree!(18, "Iterable<CSVRecord>"), // type
        tree!(18, "Closeable"), // type
      ]),
    ]),
    tree!(19; [ // type_body
      tree!(2, "    \\n     * Creates a parser for the given {@link File}.\\n     *\\n     * <p><strong>Note:</strong> This method internally creates a FileReader using\\n     * {@link FileReader#FileReader(java.io.File)} which in turn relies on the default encoding of the JVM that\\n     * is executing the code. If this is insufficient create a URL to the file and use\\n     * {@link #parse(URL, Charset, CSVFormat)}</p>\\n     *\\n     * @param file\\n     *            a CSV file. Must not be null.\\n     * @param charset\\n     *            A charset\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @return a new parser\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either file or format are null.\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
          tree!(7, "static"), // static
        ]),
        tree!(18, "CSVParser"), // type
        tree!(5, "parse"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "File"), // type
            tree!(5, "file"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "Charset"), // type
            tree!(5, "charset"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "file"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "file"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(31, "// Use the default Charset explicitly"), // line_comment
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "CSVParser"), // type
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "InputStreamReader"), // type
                  tree!(27; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(18, "FileInputStream"), // type
                      tree!(27; [ // argument_list
                        tree!(5, "file"), // identifier
                      ]),
                    ]),
                    tree!(5, "charset"), // identifier
                  ]),
                ]),
                tree!(5, "format"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Creates a parser for the given {@link String}.\\n     *\\n     * @param string\\n     *            a CSV string. Must not be null.\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @return a new parser\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either string or format are null.\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
          tree!(7, "static"), // static
        ]),
        tree!(18, "CSVParser"), // type
        tree!(5, "parse"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String"), // type
            tree!(5, "string"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "string"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "string"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "CSVParser"), // type
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "StringReader"), // type
                  tree!(27; [ // argument_list
                    tree!(5, "string"), // identifier
                  ]),
                ]),
                tree!(5, "format"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Creates a parser for the given URL.\\n     *\\n     * <p>\\n     * If you do not read all records from the given {@code url}, you should call {@link #close()} on the parser, unless\\n     * you close the {@code url}.\\n     * </p>\\n     *\\n     * @param url\\n     *            a URL. Must not be null.\\n     * @param charset\\n     *            the charset for the resource. Must not be null.\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @return a new parser\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either url, charset or format are null.\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
          tree!(7, "static"), // static
        ]),
        tree!(18, "CSVParser"), // type
        tree!(5, "parse"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "URL"), // type
            tree!(5, "url"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "Charset"), // type
            tree!(5, "charset"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "url"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "url"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "charset"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "charset"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "CSVParser"), // type
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "InputStreamReader"), // type
                  tree!(27; [ // argument_list
                    tree!(26; [ // method_invocation
                      tree!(5, "url"), // identifier
                      tree!(5, "openStream"), // identifier
                      tree!(27), // argument_list
                    ]),
                    tree!(5, "charset"), // identifier
                  ]),
                ]),
                tree!(5, "format"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(31, "// the following objects are shared to reduce garbage"), // line_comment
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "CSVFormat"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "format"), // identifier
        ]),
      ]),
      tree!(2, "/** A mapping of column names to column indices */"), // block_comment
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "Map<String, Integer>"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "headerMap"), // identifier
        ]),
      ]),
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "Lexer"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "lexer"), // identifier
        ]),
      ]),
      tree!(2, "/** A record buffer for getRecord(). Grows as necessary and is reused. */"), // block_comment
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "List<String>"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "record"), // identifier
          tree!(37, "="), // affectation_operator
          tree!(33; [ // object_creation_expression
            tree!(34, "new"), // new
            tree!(18, "ArrayList<String>"), // type
            tree!(27), // argument_list
          ]),
        ]),
      ]),
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
        ]),
        tree!(18, "long"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "recordNumber"), // identifier
        ]),
      ]),
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "Token"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "reusableToken"), // identifier
          tree!(37, "="), // affectation_operator
          tree!(33; [ // object_creation_expression
            tree!(34, "new"), // new
            tree!(18, "Token"), // type
            tree!(27), // argument_list
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Customized CSV parser using the given {@link CSVFormat}\\n     *\\n     * <p>\\n     * If you do not read all records from the given {@code reader}, you should call {@link #close()} on the parser,\\n     * unless you close the {@code reader}.\\n     * </p>\\n     *\\n     * @param reader\\n     *            a Reader containing CSV-formatted input. Must not be null.\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either reader or format are null.\\n     * @throws IOException\\n     *             If there is a problem reading the header or skipping the first record\\n     */"), // block_comment
      tree!(38; [ // constructor_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(5, "CSVParser"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "Reader"), // type
            tree!(5, "reader"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(39; [ // constructor_body
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "reader"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "reader"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(40; [ // assignment_expression
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "format"), // identifier
              ]),
              tree!(37, "="), // affectation_operator
              tree!(5, "format"), // identifier
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(40; [ // assignment_expression
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "lexer"), // identifier
              ]),
              tree!(37, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(18, "Lexer"), // type
                tree!(27; [ // argument_list
                  tree!(5, "format"), // identifier
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(18, "ExtendedBufferedReader"), // type
                    tree!(27; [ // argument_list
                      tree!(5, "reader"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(40; [ // assignment_expression
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "headerMap"), // identifier
              ]),
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(42, "this"), // this
                tree!(5, "initializeHeader"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
        ]),
        tree!(18, "void"), // type
        tree!(5, "addRecordValue"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "input"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(41; [ // field_access
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "reusableToken"), // identifier
                  ]),
                  tree!(5, "content"), // identifier
                ]),
                tree!(5, "toString"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
          tree!(43; [ // local_variable_declaration
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "nullString"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "format"), // identifier
                ]),
                tree!(5, "getNullString"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(5, "nullString"), // identifier
                tree!(47, "=="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "record"), // identifier
                  ]),
                  tree!(5, "add"), // identifier
                  tree!(27; [ // argument_list
                    tree!(5, "input"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "record"), // identifier
                  ]),
                  tree!(5, "add"), // identifier
                  tree!(27; [ // argument_list
                    tree!(49; [ // ternary_expression
                      tree!(26; [ // method_invocation
                        tree!(5, "input"), // identifier
                        tree!(5, "equalsIgnoreCase"), // identifier
                        tree!(27; [ // argument_list
                          tree!(5, "nullString"), // identifier
                        ]),
                      ]),
                      tree!(50, "?"), // ?
                      tree!(48, "null"), // null_literal
                      tree!(51, ":"), // :
                      tree!(5, "input"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Closes resources.\\n     *\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "void"), // type
        tree!(5, "close"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "lexer"), // identifier
                ]),
                tree!(47, "!="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "lexer"), // identifier
                  ]),
                  tree!(5, "close"), // identifier
                  tree!(27), // argument_list
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the current line number in the input stream.\\n     *\\n     * <p>\\n     * <strong>ATTENTION:</strong> If your CSV input has multi-line values, the returned number does not correspond to\\n     * the record number.\\n     * </p>\\n     *\\n     * @return current line number\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "long"), // type
        tree!(5, "getCurrentLineNumber"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(26; [ // method_invocation
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "lexer"), // identifier
              ]),
              tree!(5, "getCurrentLineNumber"), // identifier
              tree!(27), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a copy of the header map that iterates in column order.\\n     * <p>\\n     * The map keys are column names. The map values are 0-based indices.\\n     * </p>\\n     * @return a copy of the header map that iterates in column order.\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "Map<String, Integer>"), // type
        tree!(5, "getHeaderMap"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(49; [ // ternary_expression
              tree!(46; [ // binary_expression
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "headerMap"), // identifier
                ]),
                tree!(47, "=="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
              tree!(50, "?"), // ?
              tree!(48, "null"), // null_literal
              tree!(51, ":"), // :
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(18, "LinkedHashMap<String, Integer>"), // type
                tree!(27; [ // argument_list
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "headerMap"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the current record number in the input stream.\\n     *\\n     * <p>\\n     * <strong>ATTENTION:</strong> If your CSV input has multi-line values, the returned number does not correspond to\\n     * the line number.\\n     * </p>\\n     *\\n     * @return current line number\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "long"), // type
        tree!(5, "getRecordNumber"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(41; [ // field_access
              tree!(42, "this"), // this
              tree!(5, "recordNumber"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Parses the CSV input according to the given format and returns the content as a list of\\n     * {@link CSVRecord CSVRecords}.\\n     *\\n     * <p>\\n     * The returned content starts at the current parse-position in the stream.\\n     * </p>\\n     *\\n     * @return list of {@link CSVRecord CSVRecords}, may be empty\\n     * @throws IOException\\n     *             on parse error or input read-failure\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "List<CSVRecord>"), // type
        tree!(5, "getRecords"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(26; [ // method_invocation
              tree!(5, "getRecords"), // identifier
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "ArrayList<CSVRecord>"), // type
                  tree!(27), // argument_list
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Parses the CSV input according to the given format and adds the content to the collection of {@link CSVRecord\\n     * CSVRecords}.\\n     *\\n     * <p>\\n     * The returned content starts at the current parse-position in the stream.\\n     * </p>\\n     *\\n     * @param records\\n     *            The collection to add to.\\n     * @param <T> the type of collection used.\\n     * @return a collection of {@link CSVRecord CSVRecords}, may be empty\\n     * @throws IOException\\n     *             on parse error or input read-failure\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(52; [ // type_parameters
          tree!(53, "T extends Collection<CSVRecord>"), // type_parameter
        ]),
        tree!(18, "T"), // type
        tree!(5, "getRecords"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "T"), // type
            tree!(5, "records"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(18, "CSVRecord"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "rec"), // identifier
            ]),
          ]),
          tree!(54; [ // while_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(45; [ // parenthesized_expression
                  tree!(40; [ // assignment_expression
                    tree!(5, "rec"), // identifier
                    tree!(37, "="), // affectation_operator
                    tree!(26; [ // method_invocation
                      tree!(42, "this"), // this
                      tree!(5, "nextRecord"), // identifier
                      tree!(27), // argument_list
                    ]),
                  ]),
                ]),
                tree!(47, "!="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(5, "records"), // identifier
                  tree!(5, "add"), // identifier
                  tree!(27; [ // argument_list
                    tree!(5, "rec"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(5, "records"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Initializes the name to index mapping if the format defines a header.\\n     *\\n     * @return null if the format has no header.\\n     * @throws IOException if there is a problem reading the header or skipping the first record\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
        ]),
        tree!(18, "Map<String, Integer>"), // type
        tree!(5, "initializeHeader"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(18, "Map<String, Integer>"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "hdrMap"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(48, "null"), // null_literal
            ]),
          ]),
          tree!(43; [ // local_variable_declaration
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String[]"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "formatHeader"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "format"), // identifier
                ]),
                tree!(5, "getHeader"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(5, "formatHeader"), // identifier
                tree!(47, "!="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(40; [ // assignment_expression
                  tree!(5, "hdrMap"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(18, "LinkedHashMap<String, Integer>"), // type
                    tree!(27), // argument_list
                  ]),
                ]),
              ]),
              tree!(43; [ // local_variable_declaration
                tree!(18, "String[]"), // type
                tree!(36; [ // variable_declarator
                  tree!(5, "headerRecord"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(48, "null"), // null_literal
                ]),
              ]),
              tree!(44; [ // if_statement
                tree!(45; [ // parenthesized_expression
                  tree!(46; [ // binary_expression
                    tree!(41; [ // field_access
                      tree!(5, "formatHeader"), // identifier
                      tree!(5, "length"), // identifier
                    ]),
                    tree!(47, "=="), // comparison_operator
                    tree!(55, "0"), // decimal_integer_literal
                  ]),
                ]),
                tree!(24; [ // block
                  tree!(31, "// read the header from the first line of the file"), // line_comment
                  tree!(43; [ // local_variable_declaration
                    tree!(11; [ // modifiers
                      tree!(13, "final"), // final
                    ]),
                    tree!(18, "CSVRecord"), // type
                    tree!(36; [ // variable_declarator
                      tree!(5, "nextRecord"), // identifier
                      tree!(37, "="), // affectation_operator
                      tree!(26; [ // method_invocation
                        tree!(42, "this"), // this
                        tree!(5, "nextRecord"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                  ]),
                  tree!(44; [ // if_statement
                    tree!(45; [ // parenthesized_expression
                      tree!(46; [ // binary_expression
                        tree!(5, "nextRecord"), // identifier
                        tree!(47, "!="), // comparison_operator
                        tree!(48, "null"), // null_literal
                      ]),
                    ]),
                    tree!(24; [ // block
                      tree!(25; [ // expression_statement
                        tree!(40; [ // assignment_expression
                          tree!(5, "headerRecord"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(26; [ // method_invocation
                            tree!(5, "nextRecord"), // identifier
                            tree!(5, "values"), // identifier
                            tree!(27), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(24; [ // block
                  tree!(44; [ // if_statement
                    tree!(45; [ // parenthesized_expression
                      tree!(26; [ // method_invocation
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "format"), // identifier
                        ]),
                        tree!(5, "getSkipHeaderRecord"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                    tree!(24; [ // block
                      tree!(25; [ // expression_statement
                        tree!(26; [ // method_invocation
                          tree!(42, "this"), // this
                          tree!(5, "nextRecord"), // identifier
                          tree!(27), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(25; [ // expression_statement
                    tree!(40; [ // assignment_expression
                      tree!(5, "headerRecord"), // identifier
                      tree!(37, "="), // affectation_operator
                      tree!(5, "formatHeader"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(31, "// build the name to index mappings"), // line_comment
              tree!(44; [ // if_statement
                tree!(45; [ // parenthesized_expression
                  tree!(46; [ // binary_expression
                    tree!(5, "headerRecord"), // identifier
                    tree!(47, "!="), // comparison_operator
                    tree!(48, "null"), // null_literal
                  ]),
                ]),
                tree!(24; [ // block
                  tree!(56; [ // for_statement
                    tree!(43; [ // local_variable_declaration
                      tree!(18, "int"), // type
                      tree!(36; [ // variable_declarator
                        tree!(5, "i"), // identifier
                        tree!(37, "="), // affectation_operator
                        tree!(55, "0"), // decimal_integer_literal
                      ]),
                    ]),
                    tree!(46; [ // binary_expression
                      tree!(5, "i"), // identifier
                      tree!(47, "<"), // comparison_operator
                      tree!(41; [ // field_access
                        tree!(5, "headerRecord"), // identifier
                        tree!(5, "length"), // identifier
                      ]),
                    ]),
                    tree!(57; [ // update_expression
                      tree!(5, "i"), // identifier
                      tree!(58, "++"), // increment_operator
                    ]),
                    tree!(24; [ // block
                      tree!(43; [ // local_variable_declaration
                        tree!(11; [ // modifiers
                          tree!(13, "final"), // final
                        ]),
                        tree!(18, "String"), // type
                        tree!(36; [ // variable_declarator
                          tree!(5, "header"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(59; [ // array_access
                            tree!(5, "headerRecord"), // identifier
                            tree!(5, "i"), // identifier
                          ]),
                        ]),
                      ]),
                      tree!(43; [ // local_variable_declaration
                        tree!(11; [ // modifiers
                          tree!(13, "final"), // final
                        ]),
                        tree!(18, "boolean"), // type
                        tree!(36; [ // variable_declarator
                          tree!(5, "containsHeader"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(26; [ // method_invocation
                            tree!(5, "hdrMap"), // identifier
                            tree!(5, "containsKey"), // identifier
                            tree!(27; [ // argument_list
                              tree!(5, "header"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(43; [ // local_variable_declaration
                        tree!(11; [ // modifiers
                          tree!(13, "final"), // final
                        ]),
                        tree!(18, "boolean"), // type
                        tree!(36; [ // variable_declarator
                          tree!(5, "emptyHeader"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(26; [ // method_invocation
                            tree!(26; [ // method_invocation
                              tree!(5, "header"), // identifier
                              tree!(5, "trim"), // identifier
                              tree!(27), // argument_list
                            ]),
                            tree!(5, "isEmpty"), // identifier
                            tree!(27), // argument_list
                          ]),
                        ]),
                      ]),
                      tree!(44; [ // if_statement
                        tree!(45; [ // parenthesized_expression
                          tree!(46; [ // binary_expression
                            tree!(5, "containsHeader"), // identifier
                            tree!(60, "&&"), // logical_operator
                            tree!(45; [ // parenthesized_expression
                              tree!(46; [ // binary_expression
                                tree!(61; [ // unary_expression
                                  tree!(62, "!"), // !
                                  tree!(5, "emptyHeader"), // identifier
                                ]),
                                tree!(60, "||"), // logical_operator
                                tree!(45; [ // parenthesized_expression
                                  tree!(46; [ // binary_expression
                                    tree!(5, "emptyHeader"), // identifier
                                    tree!(60, "&&"), // logical_operator
                                    tree!(61; [ // unary_expression
                                      tree!(62, "!"), // !
                                      tree!(26; [ // method_invocation
                                        tree!(41; [ // field_access
                                          tree!(42, "this"), // this
                                          tree!(5, "format"), // identifier
                                        ]),
                                        tree!(5, "getIgnoreEmptyHeaders"), // identifier
                                        tree!(27), // argument_list
                                      ]),
                                    ]),
                                  ]),
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(24; [ // block
                          tree!(63; [ // throw_statement
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(18, "IllegalArgumentException"), // type
                              tree!(27; [ // argument_list
                                tree!(46; [ // binary_expression
                                  tree!(46; [ // binary_expression
                                    tree!(46; [ // binary_expression
                                      tree!(28; [ // string_literal
                                        tree!(29, "\""), // "
                                        tree!(30, "The header contains a duplicate name: "), // string_fragment
                                        tree!(64, "\\\""), // escape_sequence
                                        tree!(29, "\""), // "
                                      ]),
                                      tree!(9, "+"), // arithmetic_operator
                                      tree!(5, "header"), // identifier
                                    ]),
                                    tree!(9, "+"), // arithmetic_operator
                                    tree!(28; [ // string_literal
                                      tree!(29, "\""), // "
                                      tree!(64, "\\\""), // escape_sequence
                                      tree!(30, " in "), // string_fragment
                                      tree!(29, "\""), // "
                                    ]),
                                  ]),
                                  tree!(9, "+"), // arithmetic_operator
                                  tree!(26; [ // method_invocation
                                    tree!(5, "Arrays"), // identifier
                                    tree!(5, "toString"), // identifier
                                    tree!(27; [ // argument_list
                                      tree!(5, "headerRecord"), // identifier
                                    ]),
                                  ]),
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(25; [ // expression_statement
                        tree!(26; [ // method_invocation
                          tree!(5, "hdrMap"), // identifier
                          tree!(5, "put"), // identifier
                          tree!(27; [ // argument_list
                            tree!(5, "header"), // identifier
                            tree!(26; [ // method_invocation
                              tree!(5, "Integer"), // identifier
                              tree!(5, "valueOf"), // identifier
                              tree!(27; [ // argument_list
                                tree!(5, "i"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(5, "hdrMap"), // identifier
          ]),
        ]),
      ]),
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "boolean"), // type
        tree!(5, "isClosed"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(26; [ // method_invocation
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "lexer"), // identifier
              ]),
              tree!(5, "isClosed"), // identifier
              tree!(27), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns an iterator on the records.\\n     *\\n     * <p>IOExceptions occurring during the iteration are wrapped in a\\n     * RuntimeException.\\n     * If the parser is closed a call to {@code next()} will throw a\\n     * NoSuchElementException.</p>\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "Iterator<CSVRecord>"), // type
        tree!(5, "iterator"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "Iterator<CSVRecord>"), // type
              tree!(27), // argument_list
              tree!(19; [ // type_body
                tree!(35; [ // field_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "private"), // visibility
                  ]),
                  tree!(18, "CSVRecord"), // type
                  tree!(36; [ // variable_declarator
                    tree!(5, "current"), // identifier
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "private"), // visibility
                  ]),
                  tree!(18, "CSVRecord"), // type
                  tree!(5, "getNextRecord"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(65; [ // try_statement
                      tree!(66, "try"), // try
                      tree!(24; [ // block
                        tree!(32; [ // return_statement
                          tree!(26; [ // method_invocation
                            tree!(41; [ // field_access
                              tree!(5, "CSVParser"), // identifier
                              tree!(42, "this"), // this
                            ]),
                            tree!(5, "nextRecord"), // identifier
                            tree!(27), // argument_list
                          ]),
                        ]),
                      ]),
                      tree!(67; [ // catch_clause
                        tree!(68, "catch"), // catch
                        tree!(69; [ // catch_formal_parameter
                          tree!(11; [ // modifiers
                            tree!(13, "final"), // final
                          ]),
                          tree!(70; [ // catch_type
                            tree!(18, "IOException"), // type
                          ]),
                          tree!(5, "e"), // identifier
                        ]),
                        tree!(24; [ // block
                          tree!(31, "// TODO: This is not great, throw an ISE instead?"), // line_comment
                          tree!(63; [ // throw_statement
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(18, "RuntimeException"), // type
                              tree!(27; [ // argument_list
                                tree!(5, "e"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "public"), // visibility
                  ]),
                  tree!(18, "boolean"), // type
                  tree!(5, "hasNext"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(26; [ // method_invocation
                          tree!(41; [ // field_access
                            tree!(5, "CSVParser"), // identifier
                            tree!(42, "this"), // this
                          ]),
                          tree!(5, "isClosed"), // identifier
                          tree!(27), // argument_list
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(32; [ // return_statement
                          tree!(71, "false"), // false
                        ]),
                      ]),
                    ]),
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(46; [ // binary_expression
                          tree!(41; [ // field_access
                            tree!(42, "this"), // this
                            tree!(5, "current"), // identifier
                          ]),
                          tree!(47, "=="), // comparison_operator
                          tree!(48, "null"), // null_literal
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(25; [ // expression_statement
                          tree!(40; [ // assignment_expression
                            tree!(41; [ // field_access
                              tree!(42, "this"), // this
                              tree!(5, "current"), // identifier
                            ]),
                            tree!(37, "="), // affectation_operator
                            tree!(26; [ // method_invocation
                              tree!(42, "this"), // this
                              tree!(5, "getNextRecord"), // identifier
                              tree!(27), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(32; [ // return_statement
                      tree!(46; [ // binary_expression
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "current"), // identifier
                        ]),
                        tree!(47, "!="), // comparison_operator
                        tree!(48, "null"), // null_literal
                      ]),
                    ]),
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "public"), // visibility
                  ]),
                  tree!(18, "CSVRecord"), // type
                  tree!(5, "next"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(26; [ // method_invocation
                          tree!(41; [ // field_access
                            tree!(5, "CSVParser"), // identifier
                            tree!(42, "this"), // this
                          ]),
                          tree!(5, "isClosed"), // identifier
                          tree!(27), // argument_list
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(63; [ // throw_statement
                          tree!(33; [ // object_creation_expression
                            tree!(34, "new"), // new
                            tree!(18, "NoSuchElementException"), // type
                            tree!(27; [ // argument_list
                              tree!(28; [ // string_literal
                                tree!(29, "\""), // "
                                tree!(30, "CSVParser has been closed"), // string_fragment
                                tree!(29, "\""), // "
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(43; [ // local_variable_declaration
                      tree!(18, "CSVRecord"), // type
                      tree!(36; [ // variable_declarator
                        tree!(5, "next"), // identifier
                        tree!(37, "="), // affectation_operator
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "current"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(25; [ // expression_statement
                      tree!(40; [ // assignment_expression
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "current"), // identifier
                        ]),
                        tree!(37, "="), // affectation_operator
                        tree!(48, "null"), // null_literal
                      ]),
                    ]),
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(46; [ // binary_expression
                          tree!(5, "next"), // identifier
                          tree!(47, "=="), // comparison_operator
                          tree!(48, "null"), // null_literal
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(31, "// hasNext() wasn't called before"), // line_comment
                        tree!(25; [ // expression_statement
                          tree!(40; [ // assignment_expression
                            tree!(5, "next"), // identifier
                            tree!(37, "="), // affectation_operator
                            tree!(26; [ // method_invocation
                              tree!(42, "this"), // this
                              tree!(5, "getNextRecord"), // identifier
                              tree!(27), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(44; [ // if_statement
                          tree!(45; [ // parenthesized_expression
                            tree!(46; [ // binary_expression
                              tree!(5, "next"), // identifier
                              tree!(47, "=="), // comparison_operator
                              tree!(48, "null"), // null_literal
                            ]),
                          ]),
                          tree!(24; [ // block
                            tree!(63; [ // throw_statement
                              tree!(33; [ // object_creation_expression
                                tree!(34, "new"), // new
                                tree!(18, "NoSuchElementException"), // type
                                tree!(27; [ // argument_list
                                  tree!(28; [ // string_literal
                                    tree!(29, "\""), // "
                                    tree!(30, "No more CSV records available"), // string_fragment
                                    tree!(29, "\""), // "
                                  ]),
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(32; [ // return_statement
                      tree!(5, "next"), // identifier
                    ]),
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "public"), // visibility
                  ]),
                  tree!(18, "void"), // type
                  tree!(5, "remove"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(63; [ // throw_statement
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(18, "UnsupportedOperationException"), // type
                        tree!(27), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Parses the next record from the current point in the stream.\\n     *\\n     * @return the record as an array of values, or <tt>null</tt> if the end of the stream has been reached\\n     * @throws IOException\\n     *             on parse error or input read-failure\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(18, "CSVRecord"), // type
        tree!(5, "nextRecord"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(18, "CSVRecord"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(48, "null"), // null_literal
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "record"), // identifier
              ]),
              tree!(5, "clear"), // identifier
              tree!(27), // argument_list
            ]),
          ]),
          tree!(43; [ // local_variable_declaration
            tree!(18, "StringBuilder"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "sb"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(48, "null"), // null_literal
            ]),
          ]),
          tree!(72; [ // do_statement
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "reusableToken"), // identifier
                  ]),
                  tree!(5, "reset"), // identifier
                  tree!(27), // argument_list
                ]),
              ]),
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "lexer"), // identifier
                  ]),
                  tree!(5, "nextToken"), // identifier
                  tree!(27; [ // argument_list
                    tree!(41; [ // field_access
                      tree!(42, "this"), // this
                      tree!(5, "reusableToken"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(73; [ // switch_expression
                tree!(74, "switch"), // switch
                tree!(45; [ // parenthesized_expression
                  tree!(41; [ // field_access
                    tree!(41; [ // field_access
                      tree!(42, "this"), // this
                      tree!(5, "reusableToken"), // identifier
                    ]),
                    tree!(5, "type"), // identifier
                  ]),
                ]),
                tree!(75; [ // switch_block
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "TOKEN"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(25; [ // expression_statement
                      tree!(26; [ // method_invocation
                        tree!(42, "this"), // this
                        tree!(5, "addRecordValue"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "EORECORD"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(25; [ // expression_statement
                      tree!(26; [ // method_invocation
                        tree!(42, "this"), // this
                        tree!(5, "addRecordValue"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "EOF"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(41; [ // field_access
                          tree!(41; [ // field_access
                            tree!(42, "this"), // this
                            tree!(5, "reusableToken"), // identifier
                          ]),
                          tree!(5, "isReady"), // identifier
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(25; [ // expression_statement
                          tree!(26; [ // method_invocation
                            tree!(42, "this"), // this
                            tree!(5, "addRecordValue"), // identifier
                            tree!(27), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "INVALID"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(63; [ // throw_statement
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(18, "IOException"), // type
                        tree!(27; [ // argument_list
                          tree!(46; [ // binary_expression
                            tree!(46; [ // binary_expression
                              tree!(28; [ // string_literal
                                tree!(29, "\""), // "
                                tree!(30, "(line "), // string_fragment
                                tree!(29, "\""), // "
                              ]),
                              tree!(9, "+"), // arithmetic_operator
                              tree!(26; [ // method_invocation
                                tree!(42, "this"), // this
                                tree!(5, "getCurrentLineNumber"), // identifier
                                tree!(27), // argument_list
                              ]),
                            ]),
                            tree!(9, "+"), // arithmetic_operator
                            tree!(28; [ // string_literal
                              tree!(29, "\""), // "
                              tree!(30, ") invalid parse sequence"), // string_fragment
                              tree!(29, "\""), // "
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "COMMENT"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(31, "// Ignored currently"), // line_comment
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(46; [ // binary_expression
                          tree!(5, "sb"), // identifier
                          tree!(47, "=="), // comparison_operator
                          tree!(48, "null"), // null_literal
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(31, "// first comment for this record"), // line_comment
                        tree!(25; [ // expression_statement
                          tree!(40; [ // assignment_expression
                            tree!(5, "sb"), // identifier
                            tree!(37, "="), // affectation_operator
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(18, "StringBuilder"), // type
                              tree!(27), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(25; [ // expression_statement
                          tree!(26; [ // method_invocation
                            tree!(5, "sb"), // identifier
                            tree!(5, "append"), // identifier
                            tree!(27; [ // argument_list
                              tree!(41; [ // field_access
                                tree!(5, "Constants"), // identifier
                                tree!(5, "LF"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(25; [ // expression_statement
                      tree!(26; [ // method_invocation
                        tree!(5, "sb"), // identifier
                        tree!(5, "append"), // identifier
                        tree!(27; [ // argument_list
                          tree!(41; [ // field_access
                            tree!(41; [ // field_access
                              tree!(42, "this"), // this
                              tree!(5, "reusableToken"), // identifier
                            ]),
                            tree!(5, "content"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(25; [ // expression_statement
                      tree!(40; [ // assignment_expression
                        tree!(41; [ // field_access
                          tree!(41; [ // field_access
                            tree!(42, "this"), // this
                            tree!(5, "reusableToken"), // identifier
                          ]),
                          tree!(5, "type"), // identifier
                        ]),
                        tree!(37, "="), // affectation_operator
                        tree!(5, "TOKEN"), // identifier
                      ]),
                    ]),
                    tree!(31, "// Read another token"), // line_comment
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(80, "default"), // default
                    ]),
                    tree!(51, ":"), // :
                    tree!(63; [ // throw_statement
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(18, "IllegalStateException"), // type
                        tree!(27; [ // argument_list
                          tree!(46; [ // binary_expression
                            tree!(28; [ // string_literal
                              tree!(29, "\""), // "
                              tree!(30, "Unexpected Token type: "), // string_fragment
                              tree!(29, "\""), // "
                            ]),
                            tree!(9, "+"), // arithmetic_operator
                            tree!(41; [ // field_access
                              tree!(41; [ // field_access
                                tree!(42, "this"), // this
                                tree!(5, "reusableToken"), // identifier
                              ]),
                              tree!(5, "type"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(41; [ // field_access
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "reusableToken"), // identifier
                  ]),
                  tree!(5, "type"), // identifier
                ]),
                tree!(47, "=="), // comparison_operator
                tree!(5, "TOKEN"), // identifier
              ]),
            ]),
          ]),
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(61; [ // unary_expression
                tree!(62, "!"), // !
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "record"), // identifier
                  ]),
                  tree!(5, "isEmpty"), // identifier
                  tree!(27), // argument_list
                ]),
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(57; [ // update_expression
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "recordNumber"), // identifier
                  ]),
                  tree!(58, "++"), // increment_operator
                ]),
              ]),
              tree!(43; [ // local_variable_declaration
                tree!(11; [ // modifiers
                  tree!(13, "final"), // final
                ]),
                tree!(18, "String"), // type
                tree!(36; [ // variable_declarator
                  tree!(5, "comment"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(49; [ // ternary_expression
                    tree!(46; [ // binary_expression
                      tree!(5, "sb"), // identifier
                      tree!(47, "=="), // comparison_operator
                      tree!(48, "null"), // null_literal
                    ]),
                    tree!(50, "?"), // ?
                    tree!(48, "null"), // null_literal
                    tree!(51, ":"), // :
                    tree!(26; [ // method_invocation
                      tree!(5, "sb"), // identifier
                      tree!(5, "toString"), // identifier
                      tree!(27), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(25; [ // expression_statement
                tree!(40; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(18, "CSVRecord"), // type
                    tree!(27; [ // argument_list
                      tree!(26; [ // method_invocation
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "record"), // identifier
                        ]),
                        tree!(5, "toArray"), // identifier
                        tree!(27; [ // argument_list
                          tree!(81; [ // array_creation_expression
                            tree!(34, "new"), // new
                            tree!(18, "String"), // type
                            tree!(82; [ // dimensions_expr
                              tree!(26; [ // method_invocation
                                tree!(41; [ // field_access
                                  tree!(42, "this"), // this
                                  tree!(5, "record"), // identifier
                                ]),
                                tree!(5, "size"), // identifier
                                tree!(27), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(41; [ // field_access
                        tree!(42, "this"), // this
                        tree!(5, "headerMap"), // identifier
                      ]),
                      tree!(5, "comment"), // identifier
                      tree!(41; [ // field_access
                        tree!(42, "this"), // this
                        tree!(5, "recordNumber"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
    ]),
  ]),
]);

    let dst_tr = tree!(1; [ // program
  tree!(2, "\\n * Licensed to the Apache Software Foundation (ASF) under one or more\\n * contributor license agreements.  See the NOTICE file distributed with\\n * this work for additional information regarding copyright ownership.\\n * The ASF licenses this file to You under the Apache License, Version 2.0\\n * (the \"License\"); you may not use this file except in compliance with\\n * the License.  You may obtain a copy of the License at\\n *\\n *      http://www.apache.org/licenses/LICENSE-2.0\\n *\\n * Unless required by applicable law or agreed to in writing, software\\n * distributed under the License is distributed on an \"AS IS\" BASIS,\\n * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\\n * See the License for the specific language governing permissions and\\n * limitations under the License.\\n */"), // block_comment
  tree!(3; [ // package_declaration
    tree!(4, "package"), // package
    tree!(5, "org.apache.commons.csv"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.Closeable"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.File"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.FileInputStream"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.FileReader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.IOException"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.InputStreamReader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.Reader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.io.StringReader"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.net.URL"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.nio.charset.Charset"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.ArrayList"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Arrays"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Collection"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Iterator"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.LinkedHashMap"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.List"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.Map"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(5, "java.util.NoSuchElementException"), // identifier
  ]),
  tree!(6; [ // import_declaration
    tree!(7, "static"), // static
    tree!(5, "org.apache.commons.csv.Token.Type"), // identifier
    tree!(8; [ // asterisk
      tree!(9, "*"), // arithmetic_operator
    ]),
  ]),
  tree!(2, "\\n * Parses CSV files according to the specified format.\\n *\\n * Because CSV appears in many different dialects, the parser supports many formats by allowing the\\n * specification of a {@link CSVFormat}.\\n *\\n * The parser works record wise. It is not possible to go back, once a record has been parsed from the input stream.\\n *\\n * <h2>Creating instances</h2>\\n * <p>\\n * There are several static factory methods that can be used to create instances for various types of resources:\\n * </p>\\n * <ul>\\n *     <li>{@link #parse(java.io.File, Charset, CSVFormat)}</li>\\n *     <li>{@link #parse(String, CSVFormat)}</li>\\n *     <li>{@link #parse(java.net.URL, java.nio.charset.Charset, CSVFormat)}</li>\\n * </ul>\\n * <p>\\n * Alternatively parsers can also be created by passing a {@link Reader} directly to the sole constructor.\\n *\\n * For those who like fluent APIs, parsers can be created using {@link CSVFormat#parse(java.io.Reader)} as a shortcut:\\n * </p>\\n * <pre>\\n * for(CSVRecord record : CSVFormat.EXCEL.parse(in)) {\\n *     ...\\n * }\\n * </pre>\\n *\\n * <h2>Parsing record wise</h2>\\n * <p>\\n * To parse a CSV input from a file, you write:\\n * </p>\\n *\\n * <pre>\\n * File csvData = new File(&quot;/path/to/csv&quot;);\\n * CSVParser parser = CSVParser.parse(csvData, CSVFormat.RFC4180);\\n * for (CSVRecord csvRecord : parser) {\\n *     ...\\n * }\\n * </pre>\\n *\\n * <p>\\n * This will read the parse the contents of the file using the\\n * <a href=\"http://tools.ietf.org/html/rfc4180\" target=\"_blank\">RFC 4180</a> format.\\n * </p>\\n *\\n * <p>\\n * To parse CSV input in a format like Excel, you write:\\n * </p>\\n *\\n * <pre>\\n * CSVParser parser = CSVParser.parse(csvData, CSVFormat.EXCEL);\\n * for (CSVRecord csvRecord : parser) {\\n *     ...\\n * }\\n * </pre>\\n *\\n * <p>\\n * If the predefined formats don't match the format at hands, custom formats can be defined. More information about\\n * customising CSVFormats is available in {@link CSVFormat CSVFormat JavaDoc}.\\n * </p>\\n *\\n * <h2>Parsing into memory</h2>\\n * <p>\\n * If parsing record wise is not desired, the contents of the input can be read completely into memory.\\n * </p>\\n *\\n * <pre>\\n * Reader in = new StringReader(&quot;a;b\\nc;d&quot;);\\n * CSVParser parser = new CSVParser(in, CSVFormat.EXCEL);\\n * List&lt;CSVRecord&gt; list = parser.getRecords();\\n * </pre>\\n *\\n * <p>\\n * There are two constraints that have to be kept in mind:\\n * </p>\\n *\\n * <ol>\\n *     <li>Parsing into memory starts at the current position of the parser. If you have already parsed records from\\n *     the input, those records will not end up in the in memory representation of your CSV data.</li>\\n *     <li>Parsing into memory may consume a lot of system resources depending on the input. For example if you're\\n *     parsing a 150MB file of CSV data the contents will be read completely into memory.</li>\\n * </ol>\\n *\\n * <h2>Notes</h2>\\n * <p>\\n * Internal parser state is completely covered by the format and the reader-state.\\n * </p>\\n *\\n * @version $Id$\\n *\\n * @see <a href=\"package-summary.html\">package documentation for more details</a>\\n */"), // block_comment
  tree!(10; [ // type_declaration
    tree!(11; [ // modifiers
      tree!(12, "public"), // visibility
      tree!(13, "final"), // final
    ]),
    tree!(14, "class"), // type_keyword
    tree!(5, "CSVParser"), // identifier
    tree!(15; [ // super_interfaces
      tree!(16, "implements"), // implements
      tree!(17; [ // type_list
        tree!(18, "Iterable<CSVRecord>"), // type
        tree!(18, "Closeable"), // type
      ]),
    ]),
    tree!(19; [ // type_body
      tree!(2, "    \\n     * Creates a parser for the given {@link File}.\\n     *\\n     * <p><strong>Note:</strong> This method internally creates a FileReader using\\n     * {@link FileReader#FileReader(java.io.File)} which in turn relies on the default encoding of the JVM that\\n     * is executing the code. If this is insufficient create a URL to the file and use\\n     * {@link #parse(URL, Charset, CSVFormat)}</p>\\n     *\\n     * @param file\\n     *            a CSV file. Must not be null.\\n     * @param charset\\n     *            A charset\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @return a new parser\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either file or format are null.\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
          tree!(7, "static"), // static
        ]),
        tree!(18, "CSVParser"), // type
        tree!(5, "parse"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "File"), // type
            tree!(5, "file"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "Charset"), // type
            tree!(5, "charset"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "file"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "file"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(31, "// Use the default Charset explicitly"), // line_comment
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "CSVParser"), // type
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "InputStreamReader"), // type
                  tree!(27; [ // argument_list
                    tree!(33; [ // object_creation_expression
                      tree!(34, "new"), // new
                      tree!(18, "FileInputStream"), // type
                      tree!(27; [ // argument_list
                        tree!(5, "file"), // identifier
                      ]),
                    ]),
                    tree!(5, "charset"), // identifier
                  ]),
                ]),
                tree!(5, "format"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Creates a parser for the given {@link String}.\\n     *\\n     * @param string\\n     *            a CSV string. Must not be null.\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @return a new parser\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either string or format are null.\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
          tree!(7, "static"), // static
        ]),
        tree!(18, "CSVParser"), // type
        tree!(5, "parse"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String"), // type
            tree!(5, "string"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "string"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "string"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "CSVParser"), // type
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "StringReader"), // type
                  tree!(27; [ // argument_list
                    tree!(5, "string"), // identifier
                  ]),
                ]),
                tree!(5, "format"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Creates a parser for the given URL.\\n     *\\n     * <p>\\n     * If you do not read all records from the given {@code url}, you should call {@link #close()} on the parser, unless\\n     * you close the {@code url}.\\n     * </p>\\n     *\\n     * @param url\\n     *            a URL. Must not be null.\\n     * @param charset\\n     *            the charset for the resource. Must not be null.\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @return a new parser\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either url, charset or format are null.\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
          tree!(7, "static"), // static
        ]),
        tree!(18, "CSVParser"), // type
        tree!(5, "parse"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "URL"), // type
            tree!(5, "url"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "Charset"), // type
            tree!(5, "charset"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "url"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "url"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "charset"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "charset"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "CSVParser"), // type
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "InputStreamReader"), // type
                  tree!(27; [ // argument_list
                    tree!(26; [ // method_invocation
                      tree!(5, "url"), // identifier
                      tree!(5, "openStream"), // identifier
                      tree!(27), // argument_list
                    ]),
                    tree!(5, "charset"), // identifier
                  ]),
                ]),
                tree!(5, "format"), // identifier
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(31, "// the following objects are shared to reduce garbage"), // line_comment
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "CSVFormat"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "format"), // identifier
        ]),
      ]),
      tree!(2, "/** A mapping of column names to column indices */"), // block_comment
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "Map<String, Integer>"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "headerMap"), // identifier
        ]),
      ]),
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "Lexer"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "lexer"), // identifier
        ]),
      ]),
      tree!(2, "/** A record buffer for getRecord(). Grows as necessary and is reused. */"), // block_comment
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "List<String>"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "record"), // identifier
          tree!(37, "="), // affectation_operator
          tree!(33; [ // object_creation_expression
            tree!(34, "new"), // new
            tree!(18, "ArrayList<String>"), // type
            tree!(27), // argument_list
          ]),
        ]),
      ]),
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
        ]),
        tree!(18, "long"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "recordNumber"), // identifier
        ]),
      ]),
      tree!(35; [ // field_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
          tree!(13, "final"), // final
        ]),
        tree!(18, "Token"), // type
        tree!(36; [ // variable_declarator
          tree!(5, "reusableToken"), // identifier
          tree!(37, "="), // affectation_operator
          tree!(33; [ // object_creation_expression
            tree!(34, "new"), // new
            tree!(18, "Token"), // type
            tree!(27), // argument_list
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Customized CSV parser using the given {@link CSVFormat}\\n     *\\n     * <p>\\n     * If you do not read all records from the given {@code reader}, you should call {@link #close()} on the parser,\\n     * unless you close the {@code reader}.\\n     * </p>\\n     *\\n     * @param reader\\n     *            a Reader containing CSV-formatted input. Must not be null.\\n     * @param format\\n     *            the CSVFormat used for CSV parsing. Must not be null.\\n     * @throws IllegalArgumentException\\n     *             If the parameters of the format are inconsistent or if either reader or format are null.\\n     * @throws IOException\\n     *             If there is a problem reading the header or skipping the first record\\n     */"), // block_comment
      tree!(38; [ // constructor_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(5, "CSVParser"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "Reader"), // type
            tree!(5, "reader"), // identifier
          ]),
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "CSVFormat"), // type
            tree!(5, "format"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(39; [ // constructor_body
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "reader"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "reader"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(5, "Assertions"), // identifier
              tree!(5, "notNull"), // identifier
              tree!(27; [ // argument_list
                tree!(5, "format"), // identifier
                tree!(28; [ // string_literal
                  tree!(29, "\""), // "
                  tree!(30, "format"), // string_fragment
                  tree!(29, "\""), // "
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(40; [ // assignment_expression
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "format"), // identifier
              ]),
              tree!(37, "="), // affectation_operator
              tree!(5, "format"), // identifier
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(40; [ // assignment_expression
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "lexer"), // identifier
              ]),
              tree!(37, "="), // affectation_operator
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(18, "Lexer"), // type
                tree!(27; [ // argument_list
                  tree!(5, "format"), // identifier
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(18, "ExtendedBufferedReader"), // type
                    tree!(27; [ // argument_list
                      tree!(5, "reader"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(40; [ // assignment_expression
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "headerMap"), // identifier
              ]),
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(42, "this"), // this
                tree!(5, "initializeHeader"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
        ]),
        tree!(18, "void"), // type
        tree!(5, "addRecordValue"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "input"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(41; [ // field_access
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "reusableToken"), // identifier
                  ]),
                  tree!(5, "content"), // identifier
                ]),
                tree!(5, "toString"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
          tree!(43; [ // local_variable_declaration
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "nullString"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "format"), // identifier
                ]),
                tree!(5, "getNullString"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(5, "nullString"), // identifier
                tree!(47, "=="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "record"), // identifier
                  ]),
                  tree!(5, "add"), // identifier
                  tree!(27; [ // argument_list
                    tree!(5, "input"), // identifier
                  ]),
                ]),
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "record"), // identifier
                  ]),
                  tree!(5, "add"), // identifier
                  tree!(27; [ // argument_list
                    tree!(49; [ // ternary_expression
                      tree!(26; [ // method_invocation
                        tree!(5, "input"), // identifier
                        tree!(5, "equalsIgnoreCase"), // identifier
                        tree!(27; [ // argument_list
                          tree!(5, "nullString"), // identifier
                        ]),
                      ]),
                      tree!(50, "?"), // ?
                      tree!(48, "null"), // null_literal
                      tree!(51, ":"), // :
                      tree!(5, "input"), // identifier
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Closes resources.\\n     *\\n     * @throws IOException\\n     *             If an I/O error occurs\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "void"), // type
        tree!(5, "close"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "lexer"), // identifier
                ]),
                tree!(47, "!="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "lexer"), // identifier
                  ]),
                  tree!(5, "close"), // identifier
                  tree!(27), // argument_list
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the current line number in the input stream.\\n     *\\n     * <p>\\n     * <strong>ATTENTION:</strong> If your CSV input has multi-line values, the returned number does not correspond to\\n     * the record number.\\n     * </p>\\n     *\\n     * @return current line number\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "long"), // type
        tree!(5, "getCurrentLineNumber"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(26; [ // method_invocation
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "lexer"), // identifier
              ]),
              tree!(5, "getCurrentLineNumber"), // identifier
              tree!(27), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns a copy of the header map that iterates in column order.\\n     * <p>\\n     * The map keys are column names. The map values are 0-based indices.\\n     * </p>\\n     * @return a copy of the header map that iterates in column order.\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "Map<String, Integer>"), // type
        tree!(5, "getHeaderMap"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(49; [ // ternary_expression
              tree!(46; [ // binary_expression
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "headerMap"), // identifier
                ]),
                tree!(47, "=="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
              tree!(50, "?"), // ?
              tree!(48, "null"), // null_literal
              tree!(51, ":"), // :
              tree!(33; [ // object_creation_expression
                tree!(34, "new"), // new
                tree!(18, "LinkedHashMap<String, Integer>"), // type
                tree!(27; [ // argument_list
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "headerMap"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns the current record number in the input stream.\\n     *\\n     * <p>\\n     * <strong>ATTENTION:</strong> If your CSV input has multi-line values, the returned number does not correspond to\\n     * the line number.\\n     * </p>\\n     *\\n     * @return current line number\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "long"), // type
        tree!(5, "getRecordNumber"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(41; [ // field_access
              tree!(42, "this"), // this
              tree!(5, "recordNumber"), // identifier
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Parses the CSV input according to the given format and returns the content as a list of\\n     * {@link CSVRecord CSVRecords}.\\n     *\\n     * <p>\\n     * The returned content starts at the current parse-position in the stream.\\n     * </p>\\n     *\\n     * @return list of {@link CSVRecord CSVRecords}, may be empty\\n     * @throws IOException\\n     *             on parse error or input read-failure\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "List<CSVRecord>"), // type
        tree!(5, "getRecords"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(26; [ // method_invocation
              tree!(5, "getRecords"), // identifier
              tree!(27; [ // argument_list
                tree!(33; [ // object_creation_expression
                  tree!(34, "new"), // new
                  tree!(18, "ArrayList<CSVRecord>"), // type
                  tree!(27), // argument_list
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Parses the CSV input according to the given format and adds the content to the collection of {@link CSVRecord\\n     * CSVRecords}.\\n     *\\n     * <p>\\n     * The returned content starts at the current parse-position in the stream.\\n     * </p>\\n     *\\n     * @param records\\n     *            The collection to add to.\\n     * @param <T> the type of collection used.\\n     * @return a collection of {@link CSVRecord CSVRecords}, may be empty\\n     * @throws IOException\\n     *             on parse error or input read-failure\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(52; [ // type_parameters
          tree!(53, "T extends Collection<CSVRecord>"), // type_parameter
        ]),
        tree!(18, "T"), // type
        tree!(5, "getRecords"), // identifier
        tree!(21; [ // formal_parameters
          tree!(22; [ // formal_parameter
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "T"), // type
            tree!(5, "records"), // identifier
          ]),
        ]),
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(18, "CSVRecord"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "rec"), // identifier
            ]),
          ]),
          tree!(54; [ // while_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(45; [ // parenthesized_expression
                  tree!(40; [ // assignment_expression
                    tree!(5, "rec"), // identifier
                    tree!(37, "="), // affectation_operator
                    tree!(26; [ // method_invocation
                      tree!(42, "this"), // this
                      tree!(5, "nextRecord"), // identifier
                      tree!(27), // argument_list
                    ]),
                  ]),
                ]),
                tree!(47, "!="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(5, "records"), // identifier
                  tree!(5, "add"), // identifier
                  tree!(27; [ // argument_list
                    tree!(5, "rec"), // identifier
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(5, "records"), // identifier
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Initializes the name to index mapping if the format defines a header.\\n     *\\n     * @return null if the format has no header.\\n     * @throws IOException if there is a problem reading the header or skipping the first record\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "private"), // visibility
        ]),
        tree!(18, "Map<String, Integer>"), // type
        tree!(5, "initializeHeader"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(18, "Map<String, Integer>"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "hdrMap"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(48, "null"), // null_literal
            ]),
          ]),
          tree!(43; [ // local_variable_declaration
            tree!(11; [ // modifiers
              tree!(13, "final"), // final
            ]),
            tree!(18, "String[]"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "formatHeader"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(26; [ // method_invocation
                tree!(41; [ // field_access
                  tree!(42, "this"), // this
                  tree!(5, "format"), // identifier
                ]),
                tree!(5, "getHeader"), // identifier
                tree!(27), // argument_list
              ]),
            ]),
          ]),
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(5, "formatHeader"), // identifier
                tree!(47, "!="), // comparison_operator
                tree!(48, "null"), // null_literal
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(40; [ // assignment_expression
                  tree!(5, "hdrMap"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(18, "LinkedHashMap<String, Integer>"), // type
                    tree!(27), // argument_list
                  ]),
                ]),
              ]),
              tree!(43; [ // local_variable_declaration
                tree!(18, "String[]"), // type
                tree!(36; [ // variable_declarator
                  tree!(5, "headerRecord"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(48, "null"), // null_literal
                ]),
              ]),
              tree!(44; [ // if_statement
                tree!(45; [ // parenthesized_expression
                  tree!(46; [ // binary_expression
                    tree!(41; [ // field_access
                      tree!(5, "formatHeader"), // identifier
                      tree!(5, "length"), // identifier
                    ]),
                    tree!(47, "=="), // comparison_operator
                    tree!(55, "0"), // decimal_integer_literal
                  ]),
                ]),
                tree!(24; [ // block
                  tree!(31, "// read the header from the first line of the file"), // line_comment
                  tree!(43; [ // local_variable_declaration
                    tree!(11; [ // modifiers
                      tree!(13, "final"), // final
                    ]),
                    tree!(18, "CSVRecord"), // type
                    tree!(36; [ // variable_declarator
                      tree!(5, "nextRecord"), // identifier
                      tree!(37, "="), // affectation_operator
                      tree!(26; [ // method_invocation
                        tree!(42, "this"), // this
                        tree!(5, "nextRecord"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                  ]),
                  tree!(44; [ // if_statement
                    tree!(45; [ // parenthesized_expression
                      tree!(46; [ // binary_expression
                        tree!(5, "nextRecord"), // identifier
                        tree!(47, "!="), // comparison_operator
                        tree!(48, "null"), // null_literal
                      ]),
                    ]),
                    tree!(24; [ // block
                      tree!(25; [ // expression_statement
                        tree!(40; [ // assignment_expression
                          tree!(5, "headerRecord"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(26; [ // method_invocation
                            tree!(5, "nextRecord"), // identifier
                            tree!(5, "values"), // identifier
                            tree!(27), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(24; [ // block
                  tree!(44; [ // if_statement
                    tree!(45; [ // parenthesized_expression
                      tree!(26; [ // method_invocation
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "format"), // identifier
                        ]),
                        tree!(5, "getSkipHeaderRecord"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                    tree!(24; [ // block
                      tree!(25; [ // expression_statement
                        tree!(26; [ // method_invocation
                          tree!(42, "this"), // this
                          tree!(5, "nextRecord"), // identifier
                          tree!(27), // argument_list
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(25; [ // expression_statement
                    tree!(40; [ // assignment_expression
                      tree!(5, "headerRecord"), // identifier
                      tree!(37, "="), // affectation_operator
                      tree!(5, "formatHeader"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(31, "// build the name to index mappings"), // line_comment
              tree!(44; [ // if_statement
                tree!(45; [ // parenthesized_expression
                  tree!(46; [ // binary_expression
                    tree!(5, "headerRecord"), // identifier
                    tree!(47, "!="), // comparison_operator
                    tree!(48, "null"), // null_literal
                  ]),
                ]),
                tree!(24; [ // block
                  tree!(56; [ // for_statement
                    tree!(43; [ // local_variable_declaration
                      tree!(18, "int"), // type
                      tree!(36; [ // variable_declarator
                        tree!(5, "i"), // identifier
                        tree!(37, "="), // affectation_operator
                        tree!(55, "0"), // decimal_integer_literal
                      ]),
                    ]),
                    tree!(46; [ // binary_expression
                      tree!(5, "i"), // identifier
                      tree!(47, "<"), // comparison_operator
                      tree!(41; [ // field_access
                        tree!(5, "headerRecord"), // identifier
                        tree!(5, "length"), // identifier
                      ]),
                    ]),
                    tree!(57; [ // update_expression
                      tree!(5, "i"), // identifier
                      tree!(58, "++"), // increment_operator
                    ]),
                    tree!(24; [ // block
                      tree!(43; [ // local_variable_declaration
                        tree!(11; [ // modifiers
                          tree!(13, "final"), // final
                        ]),
                        tree!(18, "String"), // type
                        tree!(36; [ // variable_declarator
                          tree!(5, "header"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(59; [ // array_access
                            tree!(5, "headerRecord"), // identifier
                            tree!(5, "i"), // identifier
                          ]),
                        ]),
                      ]),
                      tree!(43; [ // local_variable_declaration
                        tree!(11; [ // modifiers
                          tree!(13, "final"), // final
                        ]),
                        tree!(18, "boolean"), // type
                        tree!(36; [ // variable_declarator
                          tree!(5, "containsHeader"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(26; [ // method_invocation
                            tree!(5, "hdrMap"), // identifier
                            tree!(5, "containsKey"), // identifier
                            tree!(27; [ // argument_list
                              tree!(5, "header"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(43; [ // local_variable_declaration
                        tree!(11; [ // modifiers
                          tree!(13, "final"), // final
                        ]),
                        tree!(18, "boolean"), // type
                        tree!(36; [ // variable_declarator
                          tree!(5, "emptyHeader"), // identifier
                          tree!(37, "="), // affectation_operator
                          tree!(46; [ // binary_expression
                            tree!(46; [ // binary_expression
                              tree!(5, "header"), // identifier
                              tree!(47, "=="), // comparison_operator
                              tree!(48, "null"), // null_literal
                            ]),
                            tree!(60, "||"), // logical_operator
                            tree!(26; [ // method_invocation
                              tree!(26; [ // method_invocation
                                tree!(5, "header"), // identifier
                                tree!(5, "trim"), // identifier
                                tree!(27), // argument_list
                              ]),
                              tree!(5, "isEmpty"), // identifier
                              tree!(27), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(44; [ // if_statement
                        tree!(45; [ // parenthesized_expression
                          tree!(46; [ // binary_expression
                            tree!(5, "containsHeader"), // identifier
                            tree!(60, "&&"), // logical_operator
                            tree!(45; [ // parenthesized_expression
                              tree!(46; [ // binary_expression
                                tree!(61; [ // unary_expression
                                  tree!(62, "!"), // !
                                  tree!(5, "emptyHeader"), // identifier
                                ]),
                                tree!(60, "||"), // logical_operator
                                tree!(45; [ // parenthesized_expression
                                  tree!(46; [ // binary_expression
                                    tree!(5, "emptyHeader"), // identifier
                                    tree!(60, "&&"), // logical_operator
                                    tree!(61; [ // unary_expression
                                      tree!(62, "!"), // !
                                      tree!(26; [ // method_invocation
                                        tree!(41; [ // field_access
                                          tree!(42, "this"), // this
                                          tree!(5, "format"), // identifier
                                        ]),
                                        tree!(5, "getIgnoreEmptyHeaders"), // identifier
                                        tree!(27), // argument_list
                                      ]),
                                    ]),
                                  ]),
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                        tree!(24; [ // block
                          tree!(63; [ // throw_statement
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(18, "IllegalArgumentException"), // type
                              tree!(27; [ // argument_list
                                tree!(46; [ // binary_expression
                                  tree!(46; [ // binary_expression
                                    tree!(46; [ // binary_expression
                                      tree!(28; [ // string_literal
                                        tree!(29, "\""), // "
                                        tree!(30, "The header contains a duplicate name: "), // string_fragment
                                        tree!(64, "\\\""), // escape_sequence
                                        tree!(29, "\""), // "
                                      ]),
                                      tree!(9, "+"), // arithmetic_operator
                                      tree!(5, "header"), // identifier
                                    ]),
                                    tree!(9, "+"), // arithmetic_operator
                                    tree!(28; [ // string_literal
                                      tree!(29, "\""), // "
                                      tree!(64, "\\\""), // escape_sequence
                                      tree!(30, " in "), // string_fragment
                                      tree!(29, "\""), // "
                                    ]),
                                  ]),
                                  tree!(9, "+"), // arithmetic_operator
                                  tree!(26; [ // method_invocation
                                    tree!(5, "Arrays"), // identifier
                                    tree!(5, "toString"), // identifier
                                    tree!(27; [ // argument_list
                                      tree!(5, "headerRecord"), // identifier
                                    ]),
                                  ]),
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(25; [ // expression_statement
                        tree!(26; [ // method_invocation
                          tree!(5, "hdrMap"), // identifier
                          tree!(5, "put"), // identifier
                          tree!(27; [ // argument_list
                            tree!(5, "header"), // identifier
                            tree!(26; [ // method_invocation
                              tree!(5, "Integer"), // identifier
                              tree!(5, "valueOf"), // identifier
                              tree!(27; [ // argument_list
                                tree!(5, "i"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(5, "hdrMap"), // identifier
          ]),
        ]),
      ]),
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "boolean"), // type
        tree!(5, "isClosed"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(26; [ // method_invocation
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "lexer"), // identifier
              ]),
              tree!(5, "isClosed"), // identifier
              tree!(27), // argument_list
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Returns an iterator on the records.\\n     *\\n     * <p>IOExceptions occurring during the iteration are wrapped in a\\n     * RuntimeException.\\n     * If the parser is closed a call to {@code next()} will throw a\\n     * NoSuchElementException.</p>\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(11; [ // modifiers
          tree!(12, "public"), // visibility
        ]),
        tree!(18, "Iterator<CSVRecord>"), // type
        tree!(5, "iterator"), // identifier
        tree!(21), // formal_parameters
        tree!(24; [ // block
          tree!(32; [ // return_statement
            tree!(33; [ // object_creation_expression
              tree!(34, "new"), // new
              tree!(18, "Iterator<CSVRecord>"), // type
              tree!(27), // argument_list
              tree!(19; [ // type_body
                tree!(35; [ // field_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "private"), // visibility
                  ]),
                  tree!(18, "CSVRecord"), // type
                  tree!(36; [ // variable_declarator
                    tree!(5, "current"), // identifier
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "private"), // visibility
                  ]),
                  tree!(18, "CSVRecord"), // type
                  tree!(5, "getNextRecord"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(65; [ // try_statement
                      tree!(66, "try"), // try
                      tree!(24; [ // block
                        tree!(32; [ // return_statement
                          tree!(26; [ // method_invocation
                            tree!(41; [ // field_access
                              tree!(5, "CSVParser"), // identifier
                              tree!(42, "this"), // this
                            ]),
                            tree!(5, "nextRecord"), // identifier
                            tree!(27), // argument_list
                          ]),
                        ]),
                      ]),
                      tree!(67; [ // catch_clause
                        tree!(68, "catch"), // catch
                        tree!(69; [ // catch_formal_parameter
                          tree!(11; [ // modifiers
                            tree!(13, "final"), // final
                          ]),
                          tree!(70; [ // catch_type
                            tree!(18, "IOException"), // type
                          ]),
                          tree!(5, "e"), // identifier
                        ]),
                        tree!(24; [ // block
                          tree!(31, "// TODO: This is not great, throw an ISE instead?"), // line_comment
                          tree!(63; [ // throw_statement
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(18, "RuntimeException"), // type
                              tree!(27; [ // argument_list
                                tree!(5, "e"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "public"), // visibility
                  ]),
                  tree!(18, "boolean"), // type
                  tree!(5, "hasNext"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(26; [ // method_invocation
                          tree!(41; [ // field_access
                            tree!(5, "CSVParser"), // identifier
                            tree!(42, "this"), // this
                          ]),
                          tree!(5, "isClosed"), // identifier
                          tree!(27), // argument_list
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(32; [ // return_statement
                          tree!(71, "false"), // false
                        ]),
                      ]),
                    ]),
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(46; [ // binary_expression
                          tree!(41; [ // field_access
                            tree!(42, "this"), // this
                            tree!(5, "current"), // identifier
                          ]),
                          tree!(47, "=="), // comparison_operator
                          tree!(48, "null"), // null_literal
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(25; [ // expression_statement
                          tree!(40; [ // assignment_expression
                            tree!(41; [ // field_access
                              tree!(42, "this"), // this
                              tree!(5, "current"), // identifier
                            ]),
                            tree!(37, "="), // affectation_operator
                            tree!(26; [ // method_invocation
                              tree!(42, "this"), // this
                              tree!(5, "getNextRecord"), // identifier
                              tree!(27), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(32; [ // return_statement
                      tree!(46; [ // binary_expression
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "current"), // identifier
                        ]),
                        tree!(47, "!="), // comparison_operator
                        tree!(48, "null"), // null_literal
                      ]),
                    ]),
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "public"), // visibility
                  ]),
                  tree!(18, "CSVRecord"), // type
                  tree!(5, "next"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(26; [ // method_invocation
                          tree!(41; [ // field_access
                            tree!(5, "CSVParser"), // identifier
                            tree!(42, "this"), // this
                          ]),
                          tree!(5, "isClosed"), // identifier
                          tree!(27), // argument_list
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(63; [ // throw_statement
                          tree!(33; [ // object_creation_expression
                            tree!(34, "new"), // new
                            tree!(18, "NoSuchElementException"), // type
                            tree!(27; [ // argument_list
                              tree!(28; [ // string_literal
                                tree!(29, "\""), // "
                                tree!(30, "CSVParser has been closed"), // string_fragment
                                tree!(29, "\""), // "
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(43; [ // local_variable_declaration
                      tree!(18, "CSVRecord"), // type
                      tree!(36; [ // variable_declarator
                        tree!(5, "next"), // identifier
                        tree!(37, "="), // affectation_operator
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "current"), // identifier
                        ]),
                      ]),
                    ]),
                    tree!(25; [ // expression_statement
                      tree!(40; [ // assignment_expression
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "current"), // identifier
                        ]),
                        tree!(37, "="), // affectation_operator
                        tree!(48, "null"), // null_literal
                      ]),
                    ]),
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(46; [ // binary_expression
                          tree!(5, "next"), // identifier
                          tree!(47, "=="), // comparison_operator
                          tree!(48, "null"), // null_literal
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(31, "// hasNext() wasn't called before"), // line_comment
                        tree!(25; [ // expression_statement
                          tree!(40; [ // assignment_expression
                            tree!(5, "next"), // identifier
                            tree!(37, "="), // affectation_operator
                            tree!(26; [ // method_invocation
                              tree!(42, "this"), // this
                              tree!(5, "getNextRecord"), // identifier
                              tree!(27), // argument_list
                            ]),
                          ]),
                        ]),
                        tree!(44; [ // if_statement
                          tree!(45; [ // parenthesized_expression
                            tree!(46; [ // binary_expression
                              tree!(5, "next"), // identifier
                              tree!(47, "=="), // comparison_operator
                              tree!(48, "null"), // null_literal
                            ]),
                          ]),
                          tree!(24; [ // block
                            tree!(63; [ // throw_statement
                              tree!(33; [ // object_creation_expression
                                tree!(34, "new"), // new
                                tree!(18, "NoSuchElementException"), // type
                                tree!(27; [ // argument_list
                                  tree!(28; [ // string_literal
                                    tree!(29, "\""), // "
                                    tree!(30, "No more CSV records available"), // string_fragment
                                    tree!(29, "\""), // "
                                  ]),
                                ]),
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(32; [ // return_statement
                      tree!(5, "next"), // identifier
                    ]),
                  ]),
                ]),
                tree!(20; [ // method_declaration
                  tree!(11; [ // modifiers
                    tree!(12, "public"), // visibility
                  ]),
                  tree!(18, "void"), // type
                  tree!(5, "remove"), // identifier
                  tree!(21), // formal_parameters
                  tree!(24; [ // block
                    tree!(63; [ // throw_statement
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(18, "UnsupportedOperationException"), // type
                        tree!(27), // argument_list
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
        ]),
      ]),
      tree!(2, "    \\n     * Parses the next record from the current point in the stream.\\n     *\\n     * @return the record as an array of values, or <tt>null</tt> if the end of the stream has been reached\\n     * @throws IOException\\n     *             on parse error or input read-failure\\n     */"), // block_comment
      tree!(20; [ // method_declaration
        tree!(18, "CSVRecord"), // type
        tree!(5, "nextRecord"), // identifier
        tree!(21), // formal_parameters
        tree!(23; [ // throws
          tree!(23, "throws"), // throws
          tree!(18, "IOException"), // type
        ]),
        tree!(24; [ // block
          tree!(43; [ // local_variable_declaration
            tree!(18, "CSVRecord"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "result"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(48, "null"), // null_literal
            ]),
          ]),
          tree!(25; [ // expression_statement
            tree!(26; [ // method_invocation
              tree!(41; [ // field_access
                tree!(42, "this"), // this
                tree!(5, "record"), // identifier
              ]),
              tree!(5, "clear"), // identifier
              tree!(27), // argument_list
            ]),
          ]),
          tree!(43; [ // local_variable_declaration
            tree!(18, "StringBuilder"), // type
            tree!(36; [ // variable_declarator
              tree!(5, "sb"), // identifier
              tree!(37, "="), // affectation_operator
              tree!(48, "null"), // null_literal
            ]),
          ]),
          tree!(72; [ // do_statement
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "reusableToken"), // identifier
                  ]),
                  tree!(5, "reset"), // identifier
                  tree!(27), // argument_list
                ]),
              ]),
              tree!(25; [ // expression_statement
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "lexer"), // identifier
                  ]),
                  tree!(5, "nextToken"), // identifier
                  tree!(27; [ // argument_list
                    tree!(41; [ // field_access
                      tree!(42, "this"), // this
                      tree!(5, "reusableToken"), // identifier
                    ]),
                  ]),
                ]),
              ]),
              tree!(73; [ // switch_expression
                tree!(74, "switch"), // switch
                tree!(45; [ // parenthesized_expression
                  tree!(41; [ // field_access
                    tree!(41; [ // field_access
                      tree!(42, "this"), // this
                      tree!(5, "reusableToken"), // identifier
                    ]),
                    tree!(5, "type"), // identifier
                  ]),
                ]),
                tree!(75; [ // switch_block
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "TOKEN"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(25; [ // expression_statement
                      tree!(26; [ // method_invocation
                        tree!(42, "this"), // this
                        tree!(5, "addRecordValue"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "EORECORD"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(25; [ // expression_statement
                      tree!(26; [ // method_invocation
                        tree!(42, "this"), // this
                        tree!(5, "addRecordValue"), // identifier
                        tree!(27), // argument_list
                      ]),
                    ]),
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "EOF"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(41; [ // field_access
                          tree!(41; [ // field_access
                            tree!(42, "this"), // this
                            tree!(5, "reusableToken"), // identifier
                          ]),
                          tree!(5, "isReady"), // identifier
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(25; [ // expression_statement
                          tree!(26; [ // method_invocation
                            tree!(42, "this"), // this
                            tree!(5, "addRecordValue"), // identifier
                            tree!(27), // argument_list
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "INVALID"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(63; [ // throw_statement
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(18, "IOException"), // type
                        tree!(27; [ // argument_list
                          tree!(46; [ // binary_expression
                            tree!(46; [ // binary_expression
                              tree!(28; [ // string_literal
                                tree!(29, "\""), // "
                                tree!(30, "(line "), // string_fragment
                                tree!(29, "\""), // "
                              ]),
                              tree!(9, "+"), // arithmetic_operator
                              tree!(26; [ // method_invocation
                                tree!(42, "this"), // this
                                tree!(5, "getCurrentLineNumber"), // identifier
                                tree!(27), // argument_list
                              ]),
                            ]),
                            tree!(9, "+"), // arithmetic_operator
                            tree!(28; [ // string_literal
                              tree!(29, "\""), // "
                              tree!(30, ") invalid parse sequence"), // string_fragment
                              tree!(29, "\""), // "
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(78, "case"), // case
                      tree!(5, "COMMENT"), // identifier
                    ]),
                    tree!(51, ":"), // :
                    tree!(31, "// Ignored currently"), // line_comment
                    tree!(44; [ // if_statement
                      tree!(45; [ // parenthesized_expression
                        tree!(46; [ // binary_expression
                          tree!(5, "sb"), // identifier
                          tree!(47, "=="), // comparison_operator
                          tree!(48, "null"), // null_literal
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(31, "// first comment for this record"), // line_comment
                        tree!(25; [ // expression_statement
                          tree!(40; [ // assignment_expression
                            tree!(5, "sb"), // identifier
                            tree!(37, "="), // affectation_operator
                            tree!(33; [ // object_creation_expression
                              tree!(34, "new"), // new
                              tree!(18, "StringBuilder"), // type
                              tree!(27), // argument_list
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(24; [ // block
                        tree!(25; [ // expression_statement
                          tree!(26; [ // method_invocation
                            tree!(5, "sb"), // identifier
                            tree!(5, "append"), // identifier
                            tree!(27; [ // argument_list
                              tree!(41; [ // field_access
                                tree!(5, "Constants"), // identifier
                                tree!(5, "LF"), // identifier
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(25; [ // expression_statement
                      tree!(26; [ // method_invocation
                        tree!(5, "sb"), // identifier
                        tree!(5, "append"), // identifier
                        tree!(27; [ // argument_list
                          tree!(41; [ // field_access
                            tree!(41; [ // field_access
                              tree!(42, "this"), // this
                              tree!(5, "reusableToken"), // identifier
                            ]),
                            tree!(5, "content"), // identifier
                          ]),
                        ]),
                      ]),
                    ]),
                    tree!(25; [ // expression_statement
                      tree!(40; [ // assignment_expression
                        tree!(41; [ // field_access
                          tree!(41; [ // field_access
                            tree!(42, "this"), // this
                            tree!(5, "reusableToken"), // identifier
                          ]),
                          tree!(5, "type"), // identifier
                        ]),
                        tree!(37, "="), // affectation_operator
                        tree!(5, "TOKEN"), // identifier
                      ]),
                    ]),
                    tree!(31, "// Read another token"), // line_comment
                    tree!(79), // break_statement
                  ]),
                  tree!(76; [ // switch_block_statement_group
                    tree!(77; [ // switch_label
                      tree!(80, "default"), // default
                    ]),
                    tree!(51, ":"), // :
                    tree!(63; [ // throw_statement
                      tree!(33; [ // object_creation_expression
                        tree!(34, "new"), // new
                        tree!(18, "IllegalStateException"), // type
                        tree!(27; [ // argument_list
                          tree!(46; [ // binary_expression
                            tree!(28; [ // string_literal
                              tree!(29, "\""), // "
                              tree!(30, "Unexpected Token type: "), // string_fragment
                              tree!(29, "\""), // "
                            ]),
                            tree!(9, "+"), // arithmetic_operator
                            tree!(41; [ // field_access
                              tree!(41; [ // field_access
                                tree!(42, "this"), // this
                                tree!(5, "reusableToken"), // identifier
                              ]),
                              tree!(5, "type"), // identifier
                            ]),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
            tree!(45; [ // parenthesized_expression
              tree!(46; [ // binary_expression
                tree!(41; [ // field_access
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "reusableToken"), // identifier
                  ]),
                  tree!(5, "type"), // identifier
                ]),
                tree!(47, "=="), // comparison_operator
                tree!(5, "TOKEN"), // identifier
              ]),
            ]),
          ]),
          tree!(44; [ // if_statement
            tree!(45; [ // parenthesized_expression
              tree!(61; [ // unary_expression
                tree!(62, "!"), // !
                tree!(26; [ // method_invocation
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "record"), // identifier
                  ]),
                  tree!(5, "isEmpty"), // identifier
                  tree!(27), // argument_list
                ]),
              ]),
            ]),
            tree!(24; [ // block
              tree!(25; [ // expression_statement
                tree!(57; [ // update_expression
                  tree!(41; [ // field_access
                    tree!(42, "this"), // this
                    tree!(5, "recordNumber"), // identifier
                  ]),
                  tree!(58, "++"), // increment_operator
                ]),
              ]),
              tree!(43; [ // local_variable_declaration
                tree!(11; [ // modifiers
                  tree!(13, "final"), // final
                ]),
                tree!(18, "String"), // type
                tree!(36; [ // variable_declarator
                  tree!(5, "comment"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(49; [ // ternary_expression
                    tree!(46; [ // binary_expression
                      tree!(5, "sb"), // identifier
                      tree!(47, "=="), // comparison_operator
                      tree!(48, "null"), // null_literal
                    ]),
                    tree!(50, "?"), // ?
                    tree!(48, "null"), // null_literal
                    tree!(51, ":"), // :
                    tree!(26; [ // method_invocation
                      tree!(5, "sb"), // identifier
                      tree!(5, "toString"), // identifier
                      tree!(27), // argument_list
                    ]),
                  ]),
                ]),
              ]),
              tree!(25; [ // expression_statement
                tree!(40; [ // assignment_expression
                  tree!(5, "result"), // identifier
                  tree!(37, "="), // affectation_operator
                  tree!(33; [ // object_creation_expression
                    tree!(34, "new"), // new
                    tree!(18, "CSVRecord"), // type
                    tree!(27; [ // argument_list
                      tree!(26; [ // method_invocation
                        tree!(41; [ // field_access
                          tree!(42, "this"), // this
                          tree!(5, "record"), // identifier
                        ]),
                        tree!(5, "toArray"), // identifier
                        tree!(27; [ // argument_list
                          tree!(81; [ // array_creation_expression
                            tree!(34, "new"), // new
                            tree!(18, "String"), // type
                            tree!(82; [ // dimensions_expr
                              tree!(26; [ // method_invocation
                                tree!(41; [ // field_access
                                  tree!(42, "this"), // this
                                  tree!(5, "record"), // identifier
                                ]),
                                tree!(5, "size"), // identifier
                                tree!(27), // argument_list
                              ]),
                            ]),
                          ]),
                        ]),
                      ]),
                      tree!(41; [ // field_access
                        tree!(42, "this"), // this
                        tree!(5, "headerMap"), // identifier
                      ]),
                      tree!(5, "comment"), // identifier
                      tree!(41; [ // field_access
                        tree!(42, "this"), // this
                        tree!(5, "recordNumber"), // identifier
                      ]),
                    ]),
                  ]),
                ]),
              ]),
            ]),
          ]),
          tree!(32; [ // return_statement
            tree!(5, "result"), // identifier
          ]),
        ]),
      ]),
    ]),
  ]),
]);
    (src_tr, dst_tr)
}
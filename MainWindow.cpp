// This
#include "MainWindow.h"

// Qt
#include <QDebug>
#include <QSplitter>
#include <QFormLayout>
#include <QVBoxLayout>

/* ============================================================================
 *
 * */
MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , mNumberOfSinusConfigLines(new QComboBox())
    , mSinusConfigLines(MAX_SINUS_LINES)
    , mPlotSignal(new QCustomPlot())
    , mPlotSamples(new QCustomPlot())
    , mPlotFrequencies(new QCustomPlot())

    , mFreqToMesure(new QDoubleSpinBox())
    , mFreqToSample(new QDoubleSpinBox())
    , mNumberOfPoints(new QSpinBox())
    , mSpectralAccurancy(new QDoubleSpinBox())
{
    // Form
    QWidget* form_widget = new QWidget();
    QFormLayout* form_layout = new QFormLayout(form_widget);

    // Number of sinus config lines
    form_layout->addRow("Number of frequences", mNumberOfSinusConfigLines);
    for(qint32 i=0 ; i<MAX_SINUS_LINES ; i++)
    {
        mNumberOfSinusConfigLines->addItem(QString("%1").arg(i+1));
    }
    connect(mNumberOfSinusConfigLines, QOverload<const QString &>::of(&QComboBox::currentIndexChanged),
            this, &MainWindow::onNumberOfSinusConfigLinesChange);

    // Sinus config lines
    for(qint32 i=0 ; i<MAX_SINUS_LINES ; i++)
    {
        mSinusConfigLines[i] = new SinusConfigLine();
        form_layout->addRow("Sin:", mSinusConfigLines[i]);

        connect(mSinusConfigLines[i], &SinusConfigLine::coeffAChanged, this, &MainWindow::updatePlotSignal);
    }
    onNumberOfSinusConfigLinesChange("1");

    mFreqToMesure->setRange(0, 0xFFFFFF);
    mFreqToSample->setRange(0, 0xFFFFFF);
    mNumberOfPoints->setRange(0, 0xFFFFFF);
    mSpectralAccurancy->setRange(0, 0xFFFFFF);

    form_layout->addRow("Fm = ", mFreqToMesure);
    form_layout->addRow("Fe = ", mFreqToSample);
    form_layout->addRow(" N = ", mNumberOfPoints);
    form_layout->addRow("SA = ", mSpectralAccurancy);

    connect(mFreqToMesure, QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &MainWindow::onFmChange);
    connect(mFreqToSample, QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &MainWindow::onFeChange);
    connect(mNumberOfPoints, QOverload<int>::of(&QSpinBox::valueChanged), this, &MainWindow::onNChange);

    // Plots
    QWidget* plots_widget = new QWidget();
    plots_widget->setMinimumWidth(1024);
    QVBoxLayout* plots_layout = new QVBoxLayout(plots_widget);

    // Create the central widget
    mCentralWidget = new QSplitter();
    setCentralWidget(mCentralWidget);
    mCentralWidget->addWidget(form_widget);
    mCentralWidget->addWidget(plots_widget);

    //
    plots_layout->addWidget(mPlotSignal);
    plots_layout->addWidget(mPlotSamples);
    plots_layout->addWidget(mPlotFrequencies);


    mNumberOfPoints->setValue( 512 );

    updatePlotSignal();
}

/* ============================================================================
 *
 * */
MainWindow::~MainWindow()
{

}

/* ============================================================================
 *
 * */
void MainWindow::updatePlotSignal()
{
    const double max_T = maxSinPeriode();

    const qint32 nbOfPoints = 1024;
    const double step = (3*max_T)/nbOfPoints;

    double maxY = 0;
    QVector<double> x(nbOfPoints), y(nbOfPoints);
    for(int i=0 ; i<nbOfPoints ; ++i)
    {
        x[i] = i*step;
        y[i] = sinVal(x[i]);
        if(maxY < y[i])
        {
            maxY = y[i];
        }
    }
    maxY += 1;

    // create graph and assign data to it:
    mPlotSignal->addGraph();
    mPlotSignal->graph(0)->setData(x, y);
    // give the axes some labels:
    mPlotSignal->xAxis->setLabel("x");
    mPlotSignal->yAxis->setLabel("y");
    // set axes ranges, so we see all data:
    mPlotSignal->xAxis->setRange(0, 3*max_T);
    mPlotSignal->yAxis->setRange(-maxY, maxY);
    mPlotSignal->replot();
}

/* ============================================================================
 *
 * */
void MainWindow::updatePlotSample()
{
    int N = mNumberOfPoints->value();
    double fe = mFreqToSample->value();
    double T = 1/fe;

    const double max_T = maxSinPeriode();

    double maxY = 0;
    QVector<double> x(N);
    mSamples.resize(N);
    for(int i=0 ; i<N ; ++i)
    {
        x[i] = i*T;
        mSamples[i] = sinVal(x[i]);
        if(maxY < mSamples[i])
        {
            maxY = mSamples[i];
        }
    }
    maxY += 1;

    // create graph and assign data to it:
    mPlotSamples->addGraph();
    mPlotSamples->graph(0)->setData(x, mSamples);

    mPlotSamples->graph(0)->setScatterStyle(QCPScatterStyle(QCPScatterStyle::ssCross, 10));
    mPlotSamples->graph(0)->setLineStyle(QCPGraph::LineStyle::lsNone);

    // give the axes some labels:
    mPlotSamples->xAxis->setLabel("x");
    mPlotSamples->yAxis->setLabel("y");
    // set axes ranges, so we see all data:
    mPlotSamples->xAxis->setRange(0, 3*max_T);
    mPlotSamples->yAxis->setRange(-maxY, maxY);
    mPlotSamples->replot();


    updatePlotFrequencies();
}

/* ============================================================================
 *
 * */
void MainWindow::updatePlotFrequencies()
{
    int N = mNumberOfPoints->value();
    double fe = mFreqToSample->value();
//    double T = 1/fe;

    // Spectral accurancy
    double sa = fe/N;
    mSpectralAccurancy->setValue(sa);

    double maxY = 0;
    QVector<double> x(N), y(N);
    for(int k=0 ; k<N ; ++k)
    {
        x[k] = k*sa;
        double img = 0;
        double real = 0;
        for(int n=0 ; n<N ; ++n)
        {
            real += mSamples[n] * cos(-2 * 3.14 * k * n / N);
            img  += mSamples[n] * sin(-2 * 3.14 * k * n / N);
        }

        y[k] = sqrt( real*real + img*img );
        if(maxY < y[k])
        {
            maxY = y[k];
        }
    }

    // create graph and assign data to it:
    mPlotFrequencies->addGraph();
    mPlotFrequencies->graph(0)->setData(x, y);

    mPlotSamples->graph(0)->setScatterStyle(QCPScatterStyle(QCPScatterStyle::ssCross, 10));
    mPlotSamples->graph(0)->setLineStyle(QCPGraph::LineStyle::lsNone);


//    QCPBars *bars1 = new QCPBars(mPlotFrequencies->xAxis, mPlotFrequencies->yAxis);
//    bars1->setWidth(sa/10.0);
//    bars1->setData(x, y);
//    bars1->setPen(Qt::NoPen);
//    bars1->setBrush(QColor(10, 140, 70, 160));


    // give the axes some labels:
    mPlotFrequencies->xAxis->setLabel("x");
    mPlotFrequencies->yAxis->setLabel("y");
    // set axes ranges, so we see all data:
    mPlotFrequencies->xAxis->setRange(0, (N+1)*sa );
    mPlotFrequencies->yAxis->setRange( 0, maxY );
    mPlotFrequencies->replot();
}

/* ============================================================================
 *
 * */
double MainWindow::sinVal(double x)
{
    double val = 0;
    for(qint32 i=0 ; i<mCurrNbOfSinusConfigLines ; i++)
    {
        val += mSinusConfigLines[i]->sinVal(x);
    }
    return val;
}

/* ============================================================================
 *
 * */
double MainWindow::maxSinPeriode()
{
    double val = 0;
    for(qint32 i=0 ; i<mCurrNbOfSinusConfigLines ; i++)
    {
        if( val < mSinusConfigLines[i]->periode() )
        {
            val = mSinusConfigLines[i]->periode();
        }
    }
    return val;
}

/* ============================================================================
 *
 * */
void MainWindow::sampleSignal()
{
    updatePlotSample();
}

/* ============================================================================
 *
 * */
void MainWindow::onNumberOfSinusConfigLinesChange(const QString &text)
{
    // Disable all
    for(qint32 i=0 ; i<MAX_SINUS_LINES ; i++)
    {
        mSinusConfigLines[i]->setEnabled(false);
    }

    // Enable some
    mCurrNbOfSinusConfigLines = text.toInt();
    for(qint32 i=0 ; i<mCurrNbOfSinusConfigLines ; i++)
    {
        mSinusConfigLines[i]->setEnabled(true);
    }
}

/* ============================================================================
 *
 * */
void MainWindow::onFmChange(double fm)
{
    double fe = (2.5*fm);
    mFreqToSample->setValue(fe);
}

/* ============================================================================
 *
 * */
void MainWindow::onFeChange(double fe)
{
//    double fm = mFreqToMesure->value();
    // mNumberOfPoints->setValue( static_cast<int>(2.0 * (fe/fm)) );
    sampleSignal();
}

/* ============================================================================
 *
 * */
void MainWindow::onNChange(int v)
{
    Q_UNUSED(v);

    sampleSignal();
}

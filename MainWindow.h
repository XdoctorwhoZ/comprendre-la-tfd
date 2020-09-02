#ifndef MAINWINDOW_H
#define MAINWINDOW_H

// Qt
#include <QVector>
#include <QSpinBox>
#include <QLineEdit>
#include <QComboBox>
#include <QMainWindow>

// QCustomPlot
#include <qcustomplot.h>

// This
#include <SinusConfigLine.h>

// Maximum number of sinus lines
#define MAX_SINUS_LINES 5

//!
//!
class MainWindow : public QMainWindow
{
    Q_OBJECT

    QSplitter* mCentralWidget;

    qint32 mCurrNbOfSinusConfigLines;
    QComboBox* mNumberOfSinusConfigLines;
    QVector<SinusConfigLine*> mSinusConfigLines;

    QCustomPlot* mPlotSignal;
    QCustomPlot* mPlotSamples;
    QCustomPlot* mPlotFrequencies;

    QDoubleSpinBox* mFreqToMesure;
    QDoubleSpinBox* mFreqToSample;
    QSpinBox* mNumberOfPoints;
    QDoubleSpinBox* mSpectralAccurancy;

    QVector<double> mSamples;

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private:
    void updatePlotSignal();
    void updatePlotSample();
    void updatePlotFrequencies();

    double sinVal(double x);

    //! Return the greatest periodes among mSinusConfigLines
    double maxSinPeriode();

    //!
    void sampleSignal();

private slots:
    void onNumberOfSinusConfigLinesChange(const QString &text);

    void onFmChange(double v);
    void onFeChange(double v);
    void onNChange(int v);

};

#endif // MAINWINDOW_H

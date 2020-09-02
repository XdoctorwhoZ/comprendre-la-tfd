#ifndef SINUSCONFIGLINE_H
#define SINUSCONFIGLINE_H

// Qt
#include <QWidget>
#include <QDoubleSpinBox>

//!
//!
class SinusConfigLine : public QWidget
{
    Q_OBJECT

    QDoubleSpinBox* mCoeffA;
    QDoubleSpinBox* mCoeffB;
    QDoubleSpinBox* mFrequence;

public:
    explicit SinusConfigLine(QWidget *parent = nullptr);

    void setEnabled(bool v);

    //!
    double sinVal(double x);

    //!
    double periode();

signals:
    void coeffAChanged();

private:
    void updateCoeffA();
    void updateFrequence();

private slots:
    void onCoeffAChange(double v);
    void onCoeffBChange(double v);
    void onFrequenceChange(double v);

};

#endif // SINUSCONFIGLINE_H

// This
#include "SinusConfigLine.h"

// Qt
#include <QtMath>
#include <QHBoxLayout>

/* ============================================================================
 *
 * */
SinusConfigLine::SinusConfigLine(QWidget *parent)
    : QWidget(parent)
    , mCoeffA(new QDoubleSpinBox())
    , mCoeffB(new QDoubleSpinBox())
    , mFrequence(new QDoubleSpinBox())
{
    QHBoxLayout* layout = new QHBoxLayout(this);
    layout->addWidget(mCoeffA);
    layout->addWidget(mCoeffB);
    layout->addWidget(mFrequence);

    mCoeffA->setRange(0,0xFFFFFFF);
    mCoeffB->setRange(0,0xFFFFFFF);
    mFrequence->setRange(0,0xFFFFFFF);

    connect(mCoeffA,    QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &SinusConfigLine::onCoeffAChange);
    connect(mCoeffB,    QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &SinusConfigLine::onCoeffBChange);
    connect(mFrequence, QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &SinusConfigLine::onFrequenceChange);

    mCoeffA->setValue(1);
    mCoeffB->setValue(0);
    updateFrequence();
}

/* ============================================================================
 *
 * */
void SinusConfigLine::setEnabled(bool v)
{
    mCoeffA->setEnabled(v);
    mCoeffB->setEnabled(v);
    mFrequence->setEnabled(v);
}

/* ============================================================================
 *
 * */
double SinusConfigLine::sinVal(double x)
{
    return qSin( (mCoeffA->value() * x) + mCoeffB->value() );
}

/* ============================================================================
 *
 * */
double SinusConfigLine::periode()
{
    double T = 1.0 / mFrequence->value();
    return T;
}

/* ============================================================================
 *
 * */
void SinusConfigLine::updateCoeffA()
{
    disconnect(mCoeffA,    QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &SinusConfigLine::onCoeffAChange);
    double T = 1.0 / mFrequence->value();
    mCoeffA->setValue((2.0*M_PI) / T);
    connect(mCoeffA,    QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &SinusConfigLine::onCoeffAChange);
    emit coeffAChanged();
}

/* ============================================================================
 *
 * */
void SinusConfigLine::updateFrequence()
{
    disconnect(mFrequence, QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &SinusConfigLine::onFrequenceChange);
    double T = (2.0*M_PI) / mCoeffA->value();
    mFrequence->setValue(1.0/T);
    connect(mFrequence, QOverload<double>::of(&QDoubleSpinBox::valueChanged), this, &SinusConfigLine::onFrequenceChange);
}

/* ============================================================================
 *
 * */
void SinusConfigLine::onCoeffAChange(double v)
{
    Q_UNUSED(v);
    updateFrequence();
    emit coeffAChanged();
}

/* ============================================================================
 *
 * */
void SinusConfigLine::onCoeffBChange(double v)
{
    Q_UNUSED(v);
}

/* ============================================================================
 *
 * */
void SinusConfigLine::onFrequenceChange(double v)
{
    Q_UNUSED(v);
    updateCoeffA();
}

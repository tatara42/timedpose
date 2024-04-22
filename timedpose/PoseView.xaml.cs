
using System.Diagnostics.Metrics;
using System.Reflection.Emit;
using System.Timers;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;


namespace timedpose
{
    /// <summary>
    /// Interaction logic for PoseView.xaml
    /// </summary>
    public partial class PoseView : UserControl
    {
        List<string> imageFiles;
        List<string> LoadedImageURI = new List<string>();
        private bool isTimer = false;
        int currentImageIndex = -1;
        private DispatcherTimer countdownTimer;
        private int targetTime = 0;
        private int elapsedTime = 0;
        private int initialTargetTime;
        private bool isCountdownTimerPaused = false;

        public PoseView(List<string> imageFiles, int initialTargetTime)
        {
            InitializeComponent();
            CountDownTimerFunction(imageFiles, initialTargetTime);
            displayImage();
        }

        private void CountDownTimerFunction(List<string> imageFiles, int initialTargetTime)
        {
            try
            {
                this.imageFiles = imageFiles;
                if (initialTargetTime > 0)
                    isTimer = true;
                if (isTimer)
                {
                    this.initialTargetTime = initialTargetTime;
                    countdownTimer = new DispatcherTimer();
                    countdownTimer.Tick += CountdownTimer_Tick;
                    countdownTimer.Interval = TimeSpan.FromSeconds(1);
                }

            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 001");
            }

        }

        private void CountdownTimer_Tick(object sender, EventArgs e)
        {
            try
            {
                elapsedTime++;
                int remainingTime = targetTime - elapsedTime;
                if (remainingTime > 0)
                {
                    string countdownText = string.Format($"{remainingTime}s");
                    timerLabel.Content = countdownText;
                }
                else
                {
                    countdownTimer.Stop();
                    displayImage();
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 002");
            }   
        }
        private void HomeButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                Window window = Window.GetWindow(this);
                window.Content = new HomeView();
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 003");
            }

        }

        private void PrevButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                if (currentImageIndex > 0)
                {
                    currentImageIndex--;
                    poseImage.Source = new BitmapImage(new Uri(LoadedImageURI[currentImageIndex]));
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 008");
            }
        }
        private void NextButton_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                if (currentImageIndex < LoadedImageURI.Count - 1)
                {
                    currentImageIndex++;
                    poseImage.Source = new BitmapImage(new Uri(LoadedImageURI[currentImageIndex]));
                }
                else
                {
                    displayImage();
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 009");
            }

        }

        private void PauseButton_Click(object sender, RoutedEventArgs e)
        {
            // Perform actions when the pause button is clicked
            // For example, you can pause a timer or perform some other actions
            try
            {
                if (isCountdownTimerPaused)
                {
                    countdownTimer.Start();
                    isCountdownTimerPaused = false;
                }
                else
                {
                    countdownTimer.Stop();
                    isCountdownTimerPaused = true;
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 004", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }

        private void poseImage_MouseDown(object sender, MouseButtonEventArgs e)
        {
            displayImage();
      
        }

        private void ShowImageButton_Click(object sender, RoutedEventArgs e)
        {
            try 
            { 
                System.Diagnostics.Process.Start("explorer.exe", LoadedImageURI[currentImageIndex]);
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 005");
            }
        }

        private void displayImage()
        {
            try
            {

                currentImageIndex++;
                Random random = new Random();
                var imageUrl = imageFiles[random.Next(0, imageFiles.Count)];
                poseImage.Source = new BitmapImage(new Uri(imageUrl));
                LoadedImageURI.Add(imageUrl);
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 006");
            }
            
            countDownTimerEl();

        }
        private void countDownTimerEl()
        {
            try
            {
                if (isTimer)
                {
                    countdownTimer.Stop();
                    elapsedTime = 0;
                    targetTime = initialTargetTime;
                    timerLabel.Content = targetTime + "s";
                    countdownTimer.Start();
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show(ex.Message, "ERROR 007");
            }

        }
        
    }
}
